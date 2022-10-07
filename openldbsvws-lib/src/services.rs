use std::iter::Iterator;
use std::str::from_utf8;

#[cfg(feature = "pretty")]
use ansi_term::{ANSIString, ANSIStrings, Colour::Fixed, Style};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate};
use roxmltree::{Document, Node};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::associations::Association;
use crate::parsable::{Parsable, ParsingError};
#[cfg(feature = "pretty")]
use crate::prettyprint::PrettyPrintable;
use crate::{bool, child, date, name, parse, text, time};

mod private {
    pub trait Sealed {}
}

/// A location. At least one of CRS or TIPLOC is specified.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Location<'a> {
    /// The location's name.
    pub name: &'a str,
    /// The CRS code of this location.
    pub crs: Option<&'a str>,
    /// The TIPLOC code of this location.
    pub tiploc: Option<&'a str>,
}

/// Forecast types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ForecastType {
    /// This time is the estimated time of arrival.
    Estimated,
    /// This time is the actual time of arrival.
    Actual,
    /// NoReport means Darwin doesn’t know whether the service has passed through this location or not.
    /// Occurs after all information available to Darwin indicates the service should have passed through this location
    /// but no positive data has been received to confirm the movement. Takes precedence over any estimated times for
    /// public display.
    NoLog,
    /// NoLog means Darwin knows the service has passed through this location but it hasn’t received any movement
    /// reports for that service at that location. Occurs after a movement report is received at a subsequent location
    /// in the service’s schedule, converting NoReport to NoLog.
    NoReport,
    /// Delayed means that the service has an unknown delay (usually related to a train not moving) so any estimated
    /// times are uncertain and should be hidden from public display.
    Delayed,
}

/// This enum is returned by `ServiceTime::lateness`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum UserLateness {
    /// If the service is early, this is returned. Note that the duration will be negative.
    ///
    /// A service which is early by a minute or less is considered "on time". See `OnTime`.
    Early(Duration),
    /// If the service is on time, this is returned.
    /// Note that the duration can be negative.
    ///
    /// A delay or earliness of a minute or less is interpreted as "on time". This is to prevent services being marked
    /// as "late" due to signalling systems rounding the time to the nearest minute.
    ///
    /// For example, a service with a scheduled arrival of 15:00 and an actual arrival of 14:59 has "arrived on time",
    /// rather than being "1 minute early".
    OnTime(Duration),
    /// If the service is late, this is returned. The duration cannot be negative.
    ///
    /// A service which is late by a minute or less is considered "on time". See `OnTime`.
    Late(Duration)
}

/// The lateness trait provides the `lateness()` function for ServiceTime and nothing else.
/// This trait is sealed.
pub trait Lateness: private::Sealed {
    fn lateness(&self) -> UserLateness {};
}

/// A service time.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ServiceTime<'a> {
    /// The public scheduled time of arrival of this service at this location. This may be missing if this location is
    /// the service's origin.
    pub scheduled_arrival: Option<DateTime<FixedOffset>>,
    /// The public scheduled time of departure of this service at this location. This may be missing if this location is
    /// the service's final destination.
    pub scheduled_departure: Option<DateTime<FixedOffset>>,
    /// The time of arrival for this service at this location. Depending on `ForecastType`, this is either:
    /// - Estimated: a forecast (corresponds to `eta`)
    /// - Actual: the actual time this train arrived (corresponds to `ata`)
    /// - NoLog, NoReport: missing
    ///
    /// Scheduled arrival may be missing if this location is the service’s origin location or if the service
    /// is 'pick up only' (`activities` contains `StopsToTakeUpPassengers`) at this location.
    pub arrival: Option<DateTime<FixedOffset>>,
    /// The time of departure for this service at this location. Depending on `ForecastType`, this is either:
    /// - Estimated: a forecast (corresponds to `etd`)
    /// - Actual: the actual time this train has departed (corresponds to `atd`)
    /// - NoLog, NoReport: missing
    ///
    /// Scheduled departure may be missing if this location is the service's final destination or if the service
    /// is 'set down only' (`activities` contains `StopsToSetDownPassengers`) at this location.
    pub departure: Option<DateTime<FixedOffset>>,
    /// The forecast type of this location. See `arrival` and `departure`. This may be missing if this location is the
    /// service's origin.
    ///
    /// This field corresponds to `arrivalType`.
    pub arrival_forecast_type: Option<ForecastType>,
    /// The departure forecast type of this location. See `arrival` and `departure`. This may be missing if this
    /// location is the service's final destination.
    ///
    /// This field corresponds to `departureType`.
    pub departure_forecast_type: Option<ForecastType>,
    /// The arrival time source of this location. This is the internal service (usually "TRUST" or "Darwin") that
    /// provided the information.
    pub arrival_source: Option<&'a str>,
    /// The arrival time source instance of this location. These map to codes that can be retrieved through
    /// `GetSourceInstanceNames`. todo! implement GetSourceInstanceNames
    pub arrival_source_instance: Option<&'a str>,
    /// The departure time source of this location. This is the internal service (usually "TRUST" or "Darwin") that
    /// provided the information.
    pub departure_source: Option<&'a str>,
    /// The departure time source instance of this location. These map to codes that can be retrieved through
    /// `GetSourceInstanceNames`. todo! implement GetSourceInstanceNames
    pub departure_source_instance: Option<&'a str>,
}

/// Activity codes.
///
/// See [Activity Codes](https://wiki.openraildata.com//index.php?title=Activity_codes) on the
/// Open Rail Data Wiki.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activity {
    /// Stops to detach vehicles. (-D)
    StopDetach,
    /// Stops to attach and detach vehicles. (-T)
    StopAttachDetach,
    /// Stops to attach vehicles. (-U)
    StopAttach,
    /// Stops or shunts for other trains to pass. (A)
    StopOrShuntForPass,
    /// Attaches or detaches an assisting locomotive. (AE)
    AttachOrDetachAssistingLocomotive,
    /// Shows as 'X' on arrival. (AX)
    ///
    /// Nor the Open Rail Data Wiki nor the CIF User Spec provide useful information on this.
    ShowsAsXOnArrival,
    /// Stops for banking locomotive. (BL)
    StopsForBankingLocomotive,
    /// Stops to change train crew. (C)
    StopsToChangeCrew,
    /// Stops to set down passengers. (D)
    ///
    /// Passengers may not board here.
    StopsToSetDownPassengers,
    /// Stops for examination. (E)
    StopsForExamination,
    /// GBPRTT (Great British Railways Transition Team) Data to add. (G)
    GBPRTTDataToAdd,
    /// Notional activity to prevent WTT columns merge. (H)
    ///
    /// This can probably be safely treated as no activity.
    Notional,
    /// Notional activity to prevent WTT columns merge where 3rd column. (H)
    ///
    /// This can probably be safely treated as no activity.
    NotionalActivityThirdColumn,
    /// Passenger count point. (K)
    PassengerCountPoint,
    /// Ticket collection and examination point. (KC)
    TicketCollectionAndExaminationPoint,
    /// Ticket examination point. (KE)
    TicketExaminationPoint,
    /// Ticket examination point for first class only. (KF)
    TicketExaminationPointFirstClass,
    /// Selective ticket examination point. (KS)
    SelectiveTicketExaminationPoint,
    /// Stops to change locomotive. (L)
    StopsToChangeLocomotive,
    /// Stop not advertised. (N)
    StopNotAdvertised,
    /// Stops for other operating reasons. (OP)
    StopsForOtherReasons,
    /// Train locomotive on rear. (OR)
    TrainLocomotiveOnRear,
    /// Propelling between points shown. (PR)
    PropellingBetweenPointsShown,
    /// Stops when required. (R)
    StopsWhenRequired,
    /// Stops for reversing move or when the driver changes ends. (RM)
    StopsForReversingMove,
    /// Stops for locomotive to run round train. (RR)
    StopsForLocomotiveToRunRoundTrain,
    /// Stops for railway personnel only. (S)
    StopsForRailwayPersonnel,
    /// Stops to take up and set down passengers. (T)
    ///
    /// Passengers may board and exit the train.
    StopsToTakeUpAndSetDownPassengers,
    /// Train begins. (TB)
    TrainBegins,
    /// Train finishes. (TF)
    TrainFinishes,
    /// Activity requested for TOPS reporting purposes. (TS)
    RequestedForTOPS,
    /// Stops or passes for tablet, staff or token. (TW)
    StopsOrPassesForTabletOrStaffOrToken,
    /// Stops to take up passengers. (U)
    ///
    /// Passengers may not exit the train.
    StopsToTakeUpPassengers,
    /// Stops for watering of coaches. (W)
    StopsForWateringOfCoaches,
    /// Passes another train at crossing point on a single line. (X)
    PassesAnotherTrain,
    /// No activity.
    None,
}

/// A location in this service's schedule. Not all locations are stopped at.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ServiceLocation<'a> {
    /// The location of this stop.
    pub location: Location<'a>,
    /// Associations that happen at this stop.
    pub associations: Option<Vec<Association<'a>>>,
    /// Ad-hoc alerts about this stop. Relatively rare, normally reserved for significant and out of the ordinary events
    /// not well covered by other, more normal, disruption message options.
    pub adhoc_alerts: Option<Vec<&'a str>>,
    /// Activities that happen at this stop.
    pub activities: Option<Vec<Activity>>,
    /// The length of the train at this stop. If None, the length is unknown.
    pub length: Option<u16>,
    /// Whether the front is detached at this stop.
    pub detach_front: bool, // someone fucked up the docs for this
    /// If true, this is an operational calling location. Arrival and departure times will be
    /// working times, rather than the usual public times.
    pub operational: bool,
    /// If true, the train passes (does not stop) at this location. No arrival times will be specified and the
    /// departure times should be interpreted as working pass times. You must not imply that a passing location will
    /// be stopped at, but you may display this to, for example, indicate the progress of a service.
    ///
    /// For example, a service may call at Kings Cross, York and Glasgow and pass through Peterborough, Grantham and
    /// Darlington along the way. Peterborough, Grantham and Darlington must not be shown on any list of calling points
    /// for this service but it is permissible, for example, to indicate whether this service has passed Peterborough
    /// on time.
    pub pass: bool,
    /// If true, the service is cancelled at this location. No ETA or ETD will be provided, but an
    /// ATA or an ATD may be present.
    pub cancelled: bool,
    /// A false destination that should be displayed for this location. False destinations should be
    /// shown to the public, but are optional. False destinations help passengers choose the right option.
    ///
    /// For example, a London Paddington to Reading local service may have a false destination at Paddington of
    /// Twyford. This is to encourage passengers at Paddington to catch the faster intercity services to Reading,
    /// leaving the local service free to carry passengers between Paddington and Twyford. Locations after
    /// Paddington will show the true destination of Reading for the local service.
    pub false_destination: Option<Location<'a>>,
    /// The platform number that the service is expected to use at this location. If None, the
    /// platform is not known.
    pub platform: Option<u8>,
    /// If true, the platform number should not be displayed to the public.
    pub platform_hidden: bool,
    /// If true, the service has been suppressed at this location and will not be displayed at the
    /// station.
    pub suppressed: bool,
    /// The arrival and departure time of this service.
    pub time: ServiceTime<'a>,
    /// The number of seconds that this train is late. Note that this may contain text. You should use
    /// `ServiceTime::lateness` instead.
    pub lateness: Option<&'a str>,
}

impl<'a, 'b> Parsable<'a, 'a, 'b> for ServiceLocation<'b> {
    fn parse(
        location: &Node<'a, 'a>,
        string: &'b str,
    ) -> Result<ServiceLocation<'b>, ParsingError<'b>> {
        if name!(location) != "location" {
            return Err(ParsingError::InvalidTagName("location"));
        }

        Ok(ServiceLocation {
            location: Location {
                name: text!(string, location, "locationName")?,
                crs: text!(string, location, "crs").ok(),
                tiploc: text!(string, location, "tiploc").ok(),
            },
            associations: {
                match child!(location, "associations").ok() {
                    None => None,
                    Some(associations) => {
                        let mut vec = Vec::new();

                        for node in associations.children() {
                            vec.push(Association::parse(&node, string)?)
                        }

                        Some(vec)
                    }
                }
            },
            adhoc_alerts: child!(location, "adhocAlerts")
                .ok()
                .and_then(|alert| todo!()),
            activities: {
                match text!(string, location, "activities")? {
                    "" => None,
                    activities => Some({
                        let mut ret: Vec<Activity> = Vec::new();

                        for activity in activities
                            .as_bytes()
                            .chunks(2)
                            .map(|x| from_utf8(x).unwrap())
                            .collect::<Vec<&str>>()
                        {
                            let code = match activity.trim() {
                                "-D" => Activity::StopDetach,
                                "-T" => Activity::StopAttachDetach,
                                "-U" => Activity::StopAttach,
                                "A" => Activity::StopOrShuntForPass,
                                "AE" => Activity::AttachOrDetachAssistingLocomotive,
                                "AX" => Activity::ShowsAsXOnArrival,
                                "BL" => Activity::StopsForBankingLocomotive,
                                "C" => Activity::StopsToChangeCrew,
                                "D" => Activity::StopsToSetDownPassengers,
                                "E" => Activity::StopsForExamination,
                                "G" => Activity::GBPRTTDataToAdd,
                                "H" => Activity::Notional,
                                "HH" => Activity::NotionalActivityThirdColumn,
                                "K" => Activity::PassengerCountPoint,
                                "KC" => Activity::TicketCollectionAndExaminationPoint,
                                "KE" => Activity::TicketExaminationPoint,
                                "KF" => Activity::TicketExaminationPointFirstClass,
                                "KS" => Activity::SelectiveTicketExaminationPoint,
                                "L" => Activity::StopsToChangeLocomotive,
                                "N" => Activity::StopNotAdvertised,
                                "OP" => Activity::StopsForOtherReasons,
                                "OR" => Activity::TrainLocomotiveOnRear,
                                "PR" => Activity::PropellingBetweenPointsShown,
                                "R" => Activity::StopsWhenRequired,
                                "RM" => Activity::StopsForReversingMove,
                                "RR" => Activity::StopsForLocomotiveToRunRoundTrain,
                                "S" => Activity::StopsForRailwayPersonnel,
                                "T" => Activity::StopsToTakeUpAndSetDownPassengers,
                                "TB" => Activity::TrainBegins,
                                "TF" => Activity::TrainFinishes,
                                "TS" => Activity::RequestedForTOPS,
                                "TW" => Activity::StopsOrPassesForTabletOrStaffOrToken,
                                "U" => Activity::StopsToTakeUpPassengers,
                                "W" => Activity::StopsForWateringOfCoaches,
                                "X" => Activity::PassesAnotherTrain,
                                "" => Activity::None,

                                x => return Err(ParsingError::InvalidActivity(x)),
                            };

                            ret.push(code);
                        }

                        ret.dedup_by(|a, _| *a == Activity::None);
                        ret
                    }),
                }
            },
            length: {
                match parse!(string, location, "length", u16) {
                    Ok(x) => {
                        if x == 0 {
                            None
                        } else {
                            Some(x)
                        }
                    }
                    Err(_) => None,
                }
            },
            detach_front: bool!(string, location, "detachFront", false)?,
            operational: bool!(string, location, "isOperational", false)?,
            pass: bool!(string, location, "isPass", false)?,
            cancelled: bool!(string, location, "isCancelled", false)?,
            false_destination: text!(string, location, "falseDest")
                .ok()
                .map(|name| Location {
                    name,
                    crs: None,
                    tiploc: text!(string, location, "fdTiploc").ok(),
                }),
            platform: parse!(string, location, "platform", u8).ok(),
            platform_hidden: bool!(string, location, "platformIsHidden", false)?,
            // The docs make this misspelling. Is it a mistake? Who knows!
            suppressed: bool!(string, location, "serviceIsSupressed", false)?,
            time: {
                let arrival_forecast_type: Option<ForecastType> = match text!(string, location, "arrivalType") {
                    Ok(typ) => match typ {
                        "Forecast" => Some(ForecastType::Forecast),
                        "Actual" => Some(ForecastType::Actual),
                        "NoLog" => Some(ForecastType::NoLog),
                        "NoReport" => Some(ForecastType::NoReport),
                        "Delayed" => Some(ForecastType::Delayed),

                        _ => Err(ParsingError::InvalidForecast(typ))?
                    },
                    Err(_) => None
                };

                let departure_forecast_type: Option<ForecastType> = match text!(string, location, "departureType") {
                    Ok(typ) => match typ {
                        "Forecast" => Some(ForecastType::Forecast),
                        "Actual" => Some(ForecastType::Actual),
                        "NoLog" => Some(ForecastType::NoLog),
                        "NoReport" => Some(ForecastType::NoReport),
                        "Delayed" => Some(ForecastType::Delayed),

                        _ => Err(ParsingError::InvalidForecast(typ))?
                    },
                    Err(_) => None
                };

                Ok(ServiceTime {
                    scheduled_arrival: time!(string, location, "sta").ok(),
                    scheduled_departure: time!(string, location, "std").ok(),
                    arrival: {
                        match arrival_forecast_type {
                            Some(typ) => match typ {
                                ForecastType::Estimated => time!(string, location, "eta").ok(),
                                ForecastType::Actual => time!(string, location, "ata").ok(),
                                ForecastType::NoLog => None,
                                ForecastType::NoReport => None,
                                ForecastType::Delayed => time!(string, location, "eta").ok(),
                            },
                            None => None,
                        }
                    },
                    departure: None,
                    arrival_forecast_type,
                    departure_forecast_type,
                    arrival_source: None,
                    arrival_source_instance: None,
                    departure_source: None,
                    departure_source_instance: None
                })
            }?,

            #[allow(deprecated)]
            lateness: text!(string, location, "lateness").ok(),
        }
    }
}

const GREY: u8 = 247;
const PASSED: u8 = 81;
const LIGHT_PASSED: u8 = 195;
const HERE: u8 = 155;
const LIGHT_HERE: u8 = 193;
const LATE: u8 = 220;
const LIGHT_LATE: u8 = 230;
const CANCELLED: u8 = 203;
const LIGHT_CANCELLED: u8 = 218;
const SCHEDULED: u8 = 183;
const LIGHT_SCHEDULED: u8 = 225;

const INDENT: &str = "    ";
const CIRCLE: &str = "●";
const LINE: &str = "│";
const ARROW: &str = "⟶";
const ARROW_LEFT: &str = "⟵";
const CROSS: &str = "⨯";
const DOTTED_CIRCLE: &str = "◯";
const SEMI_CIRCLE_1: &str = "◔";
const SEMI_CIRCLE_3: &str = "◕";

#[cfg(feature = "pretty")]
impl<'a> PrettyPrintable for ServiceLocation<'a> {
    fn pretty(&self) -> String {}
}

/// Details of a train service.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ServiceDetails<'b> {
    /// The time these details were generated.
    pub generated_at: DateTime<FixedOffset>,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    pub rid: &'b str,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI
    /// allocated replacement.
    pub uid: &'b str,
    /// The Retail Service ID of the service, if known.
    pub rsid: Option<&'b str>,
    /// The Train ID value (headcode) for this service.
    pub trainid: &'b str,
    /// The Scheduled Departure Data of this service.
    pub sdd: NaiveDate,
    /// If true, this is a passenger service. Non-passenger services should not be published to the
    /// public.
    pub passenger_service: bool,
    /// If true, this is a charter service.
    pub charter: bool,
    /// The category of this service.
    pub category: &'b str,
    /// The operator of this service.
    pub operator: &'b str,
    /// The operator code of this service.
    pub operator_code: &'b str,
    /// The cancellation reason, which is not always provided.
    pub cancel_reason: Option<&'b str>,
    /// The delay reason, which is not always provided.
    pub delay_reason: Option<&'b str>,
    /// If true, this service is operating in the reverse of its normal formation.
    pub reverse_formation: bool,
    /// The list of the locations in this service's schedule.
    pub locations: Vec<ServiceLocation<'b>>,
}

impl<'a, 'b> TryFrom<&'a str> for ServiceDetails<'a>
where
    'a: 'b,
{
    type Error = ParsingError<'a>;

    fn try_from(string: &'a str) -> Result<ServiceDetails<'a>, ParsingError<'a>> {
        let document =
            Document::parse(string).map_err(|e| ParsingError::XMLParseError { source: e })?;

        let details = document
            .root()
            .descendants()
            .find(|x| x.has_tag_name("GetServiceDetailsResult"))
            .ok_or(ParsingError::MissingField("GetServiceDetailsResult"))?;

        if name!(details) != "GetServiceDetailsResult" {
            return Err(ParsingError::InvalidTagName("GetServiceDetailsResult"));
        }

        let typ = text!(string, details, "serviceType")?;

        if typ != "train" {
            return Err(ParsingError::UnsupportedServiceType(typ));
        }

        Ok(ServiceDetails {
            generated_at: time!(string, details, "generatedAt")?,
            rid: text!(string, details, "rid")?,
            uid: text!(string, details, "uid")?,
            rsid: text!(string, details, "rsid").ok(),
            trainid: text!(string, details, "trainid")?,
            sdd: date!(string, details, "sdd")?,
            passenger_service: bool!(string, details, "isPassengerService", true)?,
            charter: bool!(string, details, "isCharter", false)?,
            category: text!(string, details, "category")?,
            operator: text!(string, details, "operator")?,
            operator_code: text!(string, details, "operatorCode")?,
            cancel_reason: text!(string, details, "cancelReason").ok(),
            delay_reason: text!(string, details, "delayReason").ok(),
            reverse_formation: bool!(string, details, "isReverseFormation", false)?,
            locations: {
                let mut vec = Vec::new();

                for node in child!(details, "locations")?.children() {
                    vec.push(ServiceLocation::parse(&node, string)?)
                }

                vec
            },
        })
    }
}

const PURPLE: u8 = 140;

#[cfg(feature = "pretty")]
impl<'a> PrettyPrintable for ServiceDetails<'a> {
    fn pretty(&self) -> String {
        let strings: &[ANSIString<'a>] = &[
            Style::default().paint("Service "),
            Style::default().bold().paint(self.rid),
            Fixed(GREY).paint("\nTSDB "),
            Fixed(GREY).bold().paint(self.uid),
            Fixed(GREY).paint("\nRSID "),
            Fixed(GREY).bold().paint(self.rsid.unwrap_or("unknown")),
            Fixed(GREY).paint("\nHeadcode "),
            Fixed(GREY).bold().paint(self.trainid),
            Fixed(GREY).paint("\nDeparts "),
            Fixed(PURPLE).bold().paint(self.sdd.to_string()),
            Fixed(GREY).paint("\nType "),
            Fixed(PURPLE).bold().paint(match self.category {
                // https://wiki.openraildata.com/index.php?title=CIF_Codes
                "OL" => "London Underground/Metro Service",
                "OU" => "Unadvertised Ordinary Passenger",
                "OO" => "Ordinary Passenger",
                "OS" => "Staff Train",

                "XC" => "Channel Tunnel",
                "XD" => "Sleeper",
                "XI" => "International",
                "XR" => "Motorail",
                "XU" => "Unadvertised Express",
                "XX" => "Express Passenger",
                "XZ" => "Sleeper (Domestic)",

                "BR" => "Rail replacement bus",
                "BS" => "Bus",
                "SS" => "Ship",

                "EE" => "Empty Coaching Stock (ECS)",
                "EL" => "ECS, London Underground/Metro Service",
                "ES" => "ECS and Staff",

                "JJ" => "Postal",
                "PM" => "Post Office Controlled Parcels",
                "PP" => "Parcels",
                "PV" => "Empty NPCCS",

                "DD" => "Departmental",
                "DH" => "Civil Engineer",
                "DI" => "Mechanical & Electrical Engineer",
                "DQ" => "Stores",
                "DT" => "Test",
                "DY" => "Signal & Telecommunications Engineer",

                "ZB" => "Locomotive & Brake Van",
                "ZZ" => "Light Locomotive",

                "J2" => "RfD Automotive (Components)",
                "H2" => "RfD Automotive (Vehicles)",
                "J3" => "RfD Edible Products (UK Contracts)",
                "J4" => "RfD Industrial Minerals (UK Contracts)",
                "J5" => "RfD Chemicals (UK Contracts)",
                "J6" => "RfD Building Materials (UK Contracts)",
                "J8" => "RfD General Merchandise (UK Contracts)",
                "H8" => "RfD European",
                "J9" => "RfD Freightliner (Contracts)",
                "H9" => "RfD Freightliner (Other)",

                "A0" => "Coal (Distributive)",
                "E0" => "Coal (Electricity) MGR",
                "B0" => "Coal (Other) and Nuclear",
                "B1" => "Metals",
                "B4" => "Aggregates",
                "B5" => "Domestic and Industrial Waste",
                "B6" => "Building Materials (TLF)",
                "B7" => "Petroleum Products",

                "H0" => "RfD European Channel Tunnel (Mixed Business)",
                "H1" => "RfD European Channel Tunnel Intermodal",
                "H3" => "RfD European Channel Tunnel Automotive",
                "H4" => "RfD European Channel Tunnel Contract Services",
                "H5" => "RfD European Channel Tunnel Haulmark",
                "H6" => "RfD European Channel Tunnel Joint Venture",

                &_ => "unknown",
            }),
            Fixed(GREY).paint(format!(" ({})", self.category)),
            Fixed(GREY).paint("\nOperated by "),
            Fixed(PURPLE).bold().paint(self.operator),
            Fixed(GREY).paint(format!(" ({})", self.operator_code)),
            Style::default().paint("\n\n"),
        ];

        let mut map = String::new();

        for location in &self.locations {
            map.push_str(&location.pretty())
        }

        let mut ret = ANSIStrings(strings).to_string();
        ret.push_str(&map);

        ret
    }
}
