use std::iter::Iterator;
use std::str::from_utf8;

use chrono::{DateTime, FixedOffset, NaiveDate};
use roxmltree::{Document, Node};

use crate::associations::Association;
use crate::parsable::{Parsable, ParsingError};
use crate::{bool, child, date, name, parse, text, time};

/// A location. At least one of CRS or TIPLOC is specified.
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
#[derive(Debug, Clone)]
pub enum ForecastType {
    /// This time is the estimated time of arrival.
    Estimated,
    /// This time is the actual time of arrival.
    Actual,
}

/// A service time.
#[derive(Debug, Clone)]
pub struct ServiceTime<'a> {
    /// The public scheduled time of arrival of this service at this location.
    pub scheduled: Option<DateTime<FixedOffset>>,
    /// The time of arrival for this service at this location. If `forecast_type` is
    /// Estimated, this is an ETA. If `forecast_type` is Actual, this is an ATA.
    pub time: Option<DateTime<FixedOffset>>,
    /// Whether the time is estimated or actual.
    pub forecast_type: Option<ForecastType>,
    /// The source of the time.
    pub source: Option<&'a str>,
}

/// Activity codes.
///
/// See [Activity Codes](https://wiki.openraildata.com//index.php?title=Activity_codes) on the
/// Open Rail Data Wiki.
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
#[derive(Debug, Clone)]
pub struct ServiceLocation<'a> {
    /// The location of this stop.
    pub location: Location<'a>,
    /// Associations that happen at this stop.
    pub associations: Option<Vec<Association<'a>>>,
    /// Ad-hoc alerts about this stop.
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
    /// If true, the train passes at this location. No arrival times will be specified and the
    /// departure times should be interpreted as working pass times.
    pub pass: bool,
    /// If true, the service is cancelled at this location. No ETA or ETD will be provided, but an
    /// ATA or an ATD may be present.
    pub cancelled: bool,
    /// A false destination that should be displayed for this location. False destinations should be
    /// shown to the public.
    pub false_destination: Option<Location<'a>>,
    /// The platform number that the service is expected to use at this location. If None, the
    /// platform is not known.
    pub platform: Option<u8>,
    /// If true, the platform number should not be displayed to the public.
    pub platform_hidden: bool,
    /// If true, the service has been suppressed at this location and will not be displayed at the
    /// station.
    pub suppressed: bool,
    /// The arrival time of this service.
    pub arrival_time: Option<ServiceTime<'a>>,
    /// The departure time of this service.
    pub departure_time: Option<ServiceTime<'a>>,
    /// The lateness of this service, as given by the API. No guarantees are made about if this is
    /// parseable to an int, and sometimes it is blatantly wrong. Please calculate it yourself from
    /// the scheduled and actual times of the service.
    #[deprecated(
        note = "lateness is not guaranteed to be parseable to an int, please use scheduled/actual arrival and departure"
    )]
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
            arrival_time: {
                match time!(string, location, "sta") {
                    Ok(sta) => {
                        let forecast_type = match text!(string, location, "arrivalType")? {
                            "Actual" => ForecastType::Actual,
                            "Forecast" => ForecastType::Estimated,
                            x => return Err(ParsingError::InvalidForecast(x)),
                        };

                        Some(ServiceTime {
                            scheduled: Some(sta),
                            time: match forecast_type {
                                ForecastType::Actual => time!(string, location, "ata").ok(),
                                ForecastType::Estimated => time!(string, location, "eta").ok(),
                            },
                            forecast_type: Some(forecast_type),
                            source: text!(string, location, "arrivalSource").ok(),
                        })
                    }
                    Err(_) => None,
                }
            },
            departure_time: {
                match time!(string, location, "std") {
                    Ok(std) => {
                        let forecast_type = match text!(string, location, "departureType")? {
                            "Actual" => ForecastType::Actual,
                            "Forecast" => ForecastType::Estimated,
                            x => return Err(ParsingError::InvalidForecast(x)),
                        };

                        Some(ServiceTime {
                            scheduled: Some(std),
                            time: match forecast_type {
                                ForecastType::Actual => time!(string, location, "atd").ok(),
                                ForecastType::Estimated => time!(string, location, "etd").ok(),
                            },
                            forecast_type: Some(forecast_type),
                            source: text!(string, location, "departureSource").ok(),
                        })
                    }
                    Err(_) => None,
                }
            },

            #[allow(deprecated)]
            lateness: text!(string, location, "lateness").ok(),
        })
    }
}

/// Details of a train service.
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
