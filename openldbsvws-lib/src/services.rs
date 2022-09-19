use std::str::from_utf8;
use std::iter::Iterator;
use chrono::{DateTime, FixedOffset, NaiveDate};
use crate::associations::Association;
use crate::parsable::Parsable;
use crate::{ParsingError, Traversable};

/// A location. At least one of CRS or TIPLOC is specified.
#[derive(Debug)]
pub struct Location {
    /// The location's name.
    pub name: String,
    /// The CRS code of this location.
    pub crs: Option<String>,
    /// The TIPLOC code of this location.
    pub tiploc: Option<String>
}

/// Forecast types.
#[derive(Debug)]
pub enum ForecastType {
    /// This time is the estimated time of arrival.
    Estimated,
    /// This time is the actual time of arrival.
    Actual
}

/// A service time.
#[derive(Debug)]
pub struct ServiceTime {
    /// The public scheduled time of arrival of this service at this location.
    pub scheduled: Option<DateTime<FixedOffset>>,
    /// The time of arrival for this service at this location. If `forecast_type` is
    /// Estimated, this is an ETA. If `forecast_type` is Actual, this is an ATA.
    pub time: Option<DateTime<FixedOffset>>,
    /// Whether the time is estimated or actual.
    pub forecast_type: Option<ForecastType>,
    /// The source of the time.
    pub source: Option<String>
}

/// Activity codes.
///
/// See [Activity Codes](https://wiki.openraildata.com//index.php?title=Activity_codes) on the
/// Open Rail Data Wiki.
#[derive(Debug, PartialEq, Eq)]
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
    NotionalActivity,
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
    ActivityRequestedForTOPS,
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
    None
}

/// A location in this service's schedule. Not all locations are stopped at.
#[derive(Debug)]
pub struct ServiceLocation {
    /// The location of this stop.
    pub location: Location,
    /// Associations that happen at this stop.
    pub associations: Option<Vec<Association>>,
    /// Ad-hoc alerts about this stop.
    pub adhoc_alerts: Option<Vec<String>>,
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
    pub false_destination: Option<Location>,
    /// The platform number that the service is expected to use at this location. If None, the
    /// platform is not known.
    pub platform: Option<u8>,
    /// If true, the platform number should not be displayed to the public.
    pub platform_hidden: bool,
    /// If true, the service has been suppressed at this location and will not be displayed at the
    /// station.
    pub suppressed: bool,
    /// The arrival time of this service.
    pub arrival_time: Option<ServiceTime>,
    /// The departure time of this service.
    pub departure_time: Option<ServiceTime>,
    /// The lateness of this service, as given by the API. No guarantees are made about if this is
    /// parseable to an int, and sometimes it is blatantly wrong. Please calculate it yourself from
    /// the scheduled and actual times of the service.
    #[deprecated(note = "lateness is not guaranteed to be parseable to an int, please use scheduled/actual arrival and departure")]
    pub lateness: Option<String>
}

impl Parsable for ServiceLocation {
    fn parse(location: impl Traversable) -> Result<ServiceLocation, ParsingError> {
        if location.tag_name() != "location" {
            return Err(ParsingError::InvalidTagName {
                expected: "location",
                found: location.tag_name().parse().unwrap()
            })
        }

        Ok(
            ServiceLocation {
                location: Location {
                    name: location.child("locationName")?.get_text()?,
                    crs: location.child("crs")?.get_text().ok(),
                    tiploc: location.child("tiploc")?.get_text().ok()
                },
                associations: {
                    match location.child("associations").ok() {
                        None => {None}
                        Some(associations) => {
                            let mut vec = Vec::new();

                            for node in associations.children() {
                                vec.push(Association::parse(node)?)
                            }

                            Some(vec)
                        }
                    }
                },
                adhoc_alerts: location.child("adhocAlerts").ok().and_then(|alert| {
                    todo!()
                }),
                activities: {
                    match &*(location.child("activities")?.get_text()?) {
                        "" => {None}
                        activities => {
                            Some({
                                let mut ret: Vec<Activity> = Vec::new();

                                for activity in activities.as_bytes().chunks(2).map(|x| from_utf8(x).unwrap()).collect::<Vec<&str>>() {
                                    let code = match activity.trim() {
                                        "-D" => {Activity::StopDetach}
                                        "-T" => {Activity::StopAttachDetach}
                                        "-U" => {Activity::StopAttach}
                                        "A" => {Activity::StopOrShuntForPass}
                                        "AE" => {Activity::AttachOrDetachAssistingLocomotive}
                                        "AX" => {Activity::ShowsAsXOnArrival}
                                        "BL" => {Activity::StopsForBankingLocomotive}
                                        "C" => {Activity::StopsToChangeCrew}
                                        "D" => {Activity::StopsToSetDownPassengers}
                                        "E" => {Activity::StopsForExamination}
                                        "G" => {Activity::GBPRTTDataToAdd}
                                        "H" => {Activity::NotionalActivity}
                                        "HH" => {Activity::NotionalActivityThirdColumn}
                                        "K" => {Activity::PassengerCountPoint}
                                        "KC" => {Activity::TicketCollectionAndExaminationPoint}
                                        "KE" => {Activity::TicketExaminationPoint}
                                        "KF" => {Activity::TicketExaminationPointFirstClass}
                                        "KS" => {Activity::SelectiveTicketExaminationPoint}
                                        "L" => {Activity::StopsToChangeLocomotive}
                                        "N" => {Activity::StopNotAdvertised}
                                        "OP" => {Activity::StopsForOtherReasons}
                                        "OR" => {Activity::TrainLocomotiveOnRear}
                                        "PR" => {Activity::PropellingBetweenPointsShown}
                                        "R" => {Activity::StopsWhenRequired}
                                        "RM" => {Activity::StopsForReversingMove}
                                        "RR" => {Activity::StopsForLocomotiveToRunRoundTrain}
                                        "S" => {Activity::StopsForRailwayPersonnel}
                                        "T" => {Activity::StopsToTakeUpAndSetDownPassengers}
                                        "TB" => {Activity::TrainBegins}
                                        "TF" => {Activity::TrainFinishes}
                                        "TS" => {Activity::ActivityRequestedForTOPS}
                                        "TW" => {Activity::StopsOrPassesForTabletOrStaffOrToken}
                                        "U" => {Activity::StopsToTakeUpPassengers}
                                        "W" => {Activity::StopsForWateringOfCoaches}
                                        "X" => {Activity::PassesAnotherTrain}
                                        "" => {Activity::None}

                                        x => {return Err(ParsingError::InvalidActivity(x.parse().unwrap()))}
                                    };

                                    ret.push(code);
                                }

                                ret.dedup_by(|a, _| *a == Activity::None);
                                ret
                            })
                        }
                    }
                },
                length: {
                    match location.child("length")?.get::<u16>() {
                        Ok(x) => {
                            if x == 0 {
                                None
                            } else {
                                Some(x)
                            }
                        }
                        Err(_) => {None}
                    }
                },
                detach_front: location.child("detachFront")?.get_bool(false)?,
                operational: location.child("isOperational")?.get_bool(false)?,
                pass: location.child("isPass")?.get_bool(false)?,
                cancelled: location.child("isCancelled")?.get_bool(false)?,
                false_destination: location.child("falseDest")?.get_text().ok().map(|name| Location {
                    name,
                    crs: None,
                    tiploc: location.child("fdTiploc").ok().and_then(|x| x.get_text().ok())
                }),
                platform: location.child("platform")?.get::<u8>().ok(),
                platform_hidden: location.child("platformIsHidden")?.get_bool(false)?,
                // The docs make this misspelling. Is it a mistake? Who knows!
                suppressed: location.child("serviceIsSupressed")?.get_bool(false)?,
                arrival_time: {
                    match location.child("sta")?.get_time() {
                        Ok(sta) => {
                            let forecast_type = match &*location.child("arrivalType")?.get_text()? {
                                "Actual" => {ForecastType::Actual}
                                "Forecast" => {ForecastType::Estimated}
                                x => {return Err(ParsingError::InvalidForecast(x.parse().unwrap()))}
                            };

                            Some(
                                ServiceTime {
                                    scheduled: Some(sta),
                                    time: match forecast_type {
                                        ForecastType::Actual => {location.child("ata")?.get_time().ok()}
                                        ForecastType::Estimated => {location.child("eta")?.get_time().ok()}
                                    },
                                    forecast_type: Some(forecast_type),
                                    source: location.child("arrivalSource")?.get_text().ok()
                                }
                            )
                        }
                        Err(_) => {None}
                    }
                },
                departure_time: {
                    match location.child("std")?.get_time() {
                        Ok(std) => {
                            let forecast_type = match &*location.child("departureType")?.get_text()? {
                                "Actual" => {ForecastType::Actual}
                                "Forecast" => {ForecastType::Estimated}
                                x => {return Err(ParsingError::InvalidForecast(x.parse().unwrap()))}
                            };

                            Some(
                                ServiceTime {
                                    scheduled: Some(std),
                                    time: match forecast_type {
                                        ForecastType::Actual => {location.child("atd")?.get_time().ok()}
                                        ForecastType::Estimated => {location.child("etd")?.get_time().ok()}
                                    },
                                    forecast_type: Some(forecast_type),
                                    source: location.child("departureSource")?.get_text().ok()
                                }
                            )
                        }
                        Err(_) => {None}
                    }
                },

                #[allow(deprecated)]
                lateness: location.child("lateness")?.get_text().ok()
            }
        )
    }
}

/// Details of a train service.
#[derive(Debug)]
pub struct ServiceDetails {
    /// The time these details were generated.
    pub generated_at: DateTime<FixedOffset>,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    pub rid: String,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI
    /// allocated replacement.
    pub uid: String,
    /// The Retail Service ID of the service, if known.
    pub rsid: Option<String>,
    /// The Train ID value (headcode) for this service.
    pub trainid: String,
    /// The Scheduled Departure Data of this service.
    pub sdd: NaiveDate,
    /// If true, this is a passenger service. Non-passenger services should not be published to the
    /// public.
    pub passenger_service: bool,
    /// If true, this is a charter service.
    pub charter: bool,
    /// The category of this service.
    pub category: String,
    /// The operator of this service.
    pub operator: String,
    /// The operator code of this service.
    pub operator_code: String,
    /// The cancellation reason, which is not always provided.
    pub cancel_reason: Option<String>,
    /// The delay reason, which is not always provided.
    pub delay_reason: Option<String>,
    /// If true, this service is operating in the reverse of its normal formation.
    pub reverse_formation: bool,
    /// The list of the locations in this service's schedule.
    pub locations: Vec<ServiceLocation>
}

impl Parsable for ServiceDetails {
    fn parse(details: impl Traversable) -> Result<ServiceDetails, ParsingError> {
        if details.tag_name() != "GetServiceDetailsResult" {
            return Err(ParsingError::InvalidTagName {
                expected: "GetServiceDetailsResult",
                found: details.tag_name().parse().unwrap()
            })
        }

        let typ = details.child("serviceType")?.get_text()?;

        if typ != "train" {
            return Err(ParsingError::UnsupportedServiceType(typ.parse().unwrap()))
        }

        Ok(
            ServiceDetails {
                generated_at: details.child("generatedAt")?.get_time()?,
                rid: details.child("rid")?.get_text()?,
                uid: details.child("uid")?.get_text()?,
                rsid: details.child("rsid")?.get_text().ok(),
                trainid: details.child("trainid")?.get_text()?,
                sdd: details.child("sdd")?.get_date()?,
                passenger_service: details.child("isPassengerService")?.get_bool(true)?,
                charter: details.child("isCharter")?.get_bool(false)?,
                category: details.child("category")?.get_text()?,
                operator: details.child("operator")?.get_text()?,
                operator_code: details.child("operatorCode")?.get_text()?,
                cancel_reason: details.child("cancelReason")?.get_text().ok(),
                delay_reason: details.child("delayReason")?.get_text().ok(),
                reverse_formation: details.child("isReverseFormation")?.get_bool(false)?,
                locations: {
                    let mut vec = Vec::new();

                    for node in details.child("locations")?
                        .children() {
                        vec.push(ServiceLocation::parse(node)?)
                    }

                    vec
                }
            }
        )
    }
}
