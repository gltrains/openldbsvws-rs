use roxmltree::{Document, Node};
use core::time::Duration;
use core::str::from_utf8;
use core::str::FromStr;
use core::any::type_name;
use core::result::Result;
use chrono::{TimeZone, DateTime, FixedOffset, NaiveDate};
use thiserror::Error;

#[cfg(feature = "reqwest")]
use reqwest::Client;

// Why are these macros and not consts?
// For some reason, format! does not support
// consts.

macro_rules! service_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldbsv/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetServiceDetailsByRIDRequest><ldb:rid>{rid}</ldb:rid></ldb:GetServiceDetailsByRIDRequest></soapenv:Body></soapenv:Envelope>"}
}

macro_rules! arrival_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldb/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetArrivalBoardRequest><ldb:numRows>150</ldb:numRows><ldb:crs>{crs}</ldb:crs><ldb:filterCrs>{filter_crs}</ldb:filterCrs><ldb:filterType>{filter_type}</ldb:filterType><ldb:timeOffset>{time_offset}</ldb:timeOffset><ldb:timeWindow>{time_window}</ldb:timeWindow></ldb:GetArrivalBoardRequest></soapenv:Body></soapenv:Envelope>"}
}

#[derive(Debug)]
pub struct Location {
    name: String,
    crs: Option<String>,
    tiploc: Option<String>
}

#[derive(Debug)]
/// Train association categories.
pub enum AssociationCategory {
    /// A train joins this train.
    Join,
    /// A train divides from this train.
    Divide,
    /// A train links from this train.
    LinkedFrom,
    /// A train links to this train.
    LinkedTo
}

#[derive(Debug)]
/// A train association.
///
/// A train can join, divide, link from and link to another train.
pub struct Association {
    /// The association category.
    category: AssociationCategory,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    rid: String,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI allocated replacement.
    uid: String,
    /// The Train ID value (headcode) for this service.
    trainid: String,
    /// The Retail Service ID for this service, if known.
    rsid: Option<String>,
    /// The Scheduled Departure Date of this service.
    sdd: NaiveDate,
    /// The origin location of the associated service.
    origin: Option<Location>,
    /// The destination location of the associated service.
    destination: Option<Location>,
    /// If true, this association is cancelled and will no longer happen.
    cancelled: bool
}

#[derive(Debug)]
/// Forecast types.
pub enum ForecastType {
    /// This time is the estimated time of arrival.
    Estimated,
    /// This time is the actual time of arrival.
    Actual
}

#[derive(Debug)]
/// A service time.
pub struct ServiceTime<T: TimeZone> {
    /// The public scheduled time of arrival of this service at this location.
    scheduled: Option<DateTime<T>>,
    /// The time of arrival for this service at this location. If `forecast_type` is
    /// Estimated, this is an ETA. If `forecast_type` is Actual, this is an ATA.
    time: Option<DateTime<T>>,
    /// Whether the time is estimated or actual.
    forecast_type: Option<ForecastType>,
    /// The source of the time.
    source: Option<String>
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
    /// Train finishes. (Destination)
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

#[derive(Debug)]
/// A location in this service's schedule. Not all locations are stopped at.
pub struct ServiceLocation<T: TimeZone> {
    /// The location of this stop.
    location: Location,
    /// Associations that happen at this stop.
    associations: Option<Vec<Association>>,
    /// Ad-hoc alerts about this stop.
    adhoc_alerts: Option<Vec<String>>,
    /// Activities that happen at this stop.
    activities: Option<Vec<Activity>>,
    /// The length of the train at this stop. If None, the length is unknown.
    length: Option<u16>,
    /// Whether the front is detached at this stop.
    detach_front: bool, // someone fucked up the docs for this
    /// If true, this is an operational calling location. Arrival and departure times will be
    /// working times, rather than the usual public times.
    operational: bool,
    /// If true, the train passes at this location. No arrival times will be specified and the
    /// departure times should be interpreted as working pass times.
    pass: bool,
    /// If true, the service is cancelled at this location. No ETA or ETD will be provided, but an
    /// ATA or an ATD may be present.
    cancelled: bool,
    /// A false destination that should be displayed for this location. False destinations should be
    /// shown to the public.
    false_destination: Option<Location>,
    /// The platform number that the service is expected to use at this location. If None, the
    /// platform is not known.
    platform: Option<u8>,
    /// If true, the platform number should not be displayed to the public.
    platform_hidden: bool,
    /// If true, the service has been suppressed at this location and will not be displayed at the
    /// station.
    suppressed: bool,
    /// The arrival time of this service.
    arrival_time: Option<ServiceTime<T>>,
    /// The departure time of this service.
    departure_time: Option<ServiceTime<T>>,
    /// The lateness of this service, as given by the API. No guarantees are made about if this is
    /// parseable to an int, and sometimes it is blatantly wrong. Please calculate it yourself from
    /// the scheduled and actual times of the service.
    #[deprecated(note = "lateness is not guaranteed to be parseable to an int, please use scheduled/actual arrival and departure")]
    lateness: Option<String>
}

#[derive(Debug)]
/// Details of a train service.
pub struct ServiceDetails<T: TimeZone> {
    /// The time these details were generated.
    generated_at: DateTime<T>,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    rid: String,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI
    /// allocated replacement.
    uid: String,
    /// The Retail Service ID of the service, if known.
    rsid: Option<String>,
    /// The Train ID value (headcode) for this service.
    trainid: String,
    /// The Scheduled Departure Data of this service.
    sdd: NaiveDate,
    /// If true, this is a passenger service. Non-passenger services should not be published to the
    /// public.
    passenger_service: bool,
    /// If true, this is a charter service.
    charter: bool,
    /// The category of this service.
    category: String,
    /// The operator of this service.
    operator: String,
    /// The operator code of this service.
    operator_code: String,
    /// The cancellation reason, which is not always provided.
    cancel_reason: Option<String>,
    /// The delay reason, which is not always provided.
    delay_reason: Option<String>,
    /// If true, this service is operating in the reverse of its normal formation.
    reverse_formation: bool,
    /// The list of the locations in this service's schedule.
    locations: Vec<ServiceLocation<T>>
}

/// A parsing error.
/// This describes an error that occurred while translating XML into a struct.
#[derive(Error, Debug)]
pub enum ParsingError {
    /// An invalid tag name.
    #[error("invalid tag name, expected {expected:?}, got {found:?}")]
    InvalidTagName {
        /// The tag name that was expected.
        expected: &'static str,
        /// The tag name that was found.
        found: String
    },
    /// An invalid activity. The string represents the activity that was found.
    #[error("invalid activity, got {0}")]
    InvalidActivity(String),
    /// An invalid forecast type. The string represents the forecast type that was found.
    #[error("invalid forecast type, expected Actual or Forecast, got {0}")]
    InvalidForecast(String),
    /// An invalid association category type. The string represents the category that was found.
    #[error("invalid association category, expected Join or Divide, got {0}")]
    InvalidAssociationCategory(String),
    /// A missing field. The string represents the field that was not found.
    #[error("field {0} is missing")]
    MissingField(&'static str),
    /// An invalid field.
    #[error("field {field:?} couldn't be parsed, expected {expected:?}, got {found:?}")]
    InvalidField {
        /// The field name.
        field: &'static str,
        /// What was expected. This is purely for diagnostic reasons.
        expected: &'static str,
        /// The contents of the field.
        found: Option<String>
    },
    /// An unsupported service type. The string represents the service type that was found.
    #[error("unsupported service type {0}")]
    UnsupportedServiceType(String)
}

trait Traversable {
    fn field_text(&self, name: &'static str) -> Result<String, ParsingError>;
    fn field<T: FromStr>(&self, name: &'static str) -> Result<T, ParsingError>;
    fn field_time(&self, name: &'static str) -> Result<DateTime<FixedOffset>, ParsingError>;
    fn field_date(&self, name: &'static str)-> Result<NaiveDate, ParsingError>;
    fn field_bool(&self, name: &'static str, default: bool) -> Result<bool, ParsingError>;
}

impl<'a, 'b> Traversable for Node<'a, 'b> {
    fn field_text(&self, name: &'static str) -> Result<String, ParsingError> {
        let node = self.children()
            .find(|x| x.has_tag_name(name))
            .ok_or(ParsingError::MissingField(name))?;

        Ok(
            node.text()
                .ok_or(ParsingError::InvalidField {
                    field: name,
                    expected: "text",
                    found: None
                })?
                .to_string()
        )
    }

    fn field<T: FromStr>(&self, name: &'static str) -> Result<T, ParsingError> {
        let text = self.field_text(name)?;

        text.parse::<T>()
            .map_err(|_| ParsingError::InvalidField {
                field: name,
                expected: type_name::<T>(),
                found: Some(text)
            })
    }

    fn field_time(&self, name: &'static str) -> Result<DateTime<FixedOffset>, ParsingError> {
        let text = self.field_text(name)?;

        DateTime::parse_from_rfc3339(&text)
            .map_err(|_| ParsingError::InvalidField {
                field: name,
                expected: "DateTime",
                found: Some(text)
            })
    }

    #[inline(always)]
    fn field_date(&self, name: &'static str) -> Result<NaiveDate, ParsingError> {
        let text = self.field_text(name)?;

        NaiveDate::parse_from_str(&text, "%Y-%m-%d")
            .map_err(|_| ParsingError::InvalidField {
                field: name,
                expected: "NaiveDate",
                found: Some(text)
            })
    }

    #[inline(always)]
    fn field_bool(&self, name: &'static str, default: bool) -> Result<bool, ParsingError> {
        match self.field_text(name) {
            Ok(x) => {
                match &*x {
                    "true" => {
                        Ok(true)
                    },
                    "false" => {
                        Ok(false)
                    },
                    _ => {
                        Err(ParsingError::InvalidField {
                            field: name,
                            expected: "bool",
                            found: Some(x)
                        })
                    }
                }
            }
            Err(_) => {Ok(default)}
        }
    }
}

fn parse_association(association: Node) -> Result<Association, ParsingError> {
    if !association.has_tag_name("association") {
        return Err(ParsingError::InvalidTagName {
            expected: "association",
            found: association.tag_name().name().parse().unwrap()
        })
    }

    Ok(
        Association {
            category: match &*association.field_text("category")? {
                "divide" => {AssociationCategory::Divide}
                "join" => {AssociationCategory::Join}
                x => {return Err(ParsingError::InvalidAssociationCategory(x.parse().unwrap()))}
            },
            rid: association.field_text("rid")?,
            uid: association.field_text("uid")?,
            trainid: association.field_text("trainid")?,
            rsid: association.field_text("rsid").ok(),
            sdd: association.field_date("sdd")?,
            origin: Some(Location {
                name: association.field_text("origin")?,
                crs: association.field_text("originCRS").ok(),
                tiploc: association.field_text("originTiploc").ok()
            }),
            destination: Some(Location {
                name: association.field_text("destination")?,
                crs: association.field_text("destCRS").ok(),
                tiploc: association.field_text("destTiploc").ok()
            }),
            cancelled: association.field_bool("cancelled", false)?
        }
    )
}

fn parse_service_location(location: Node) -> Result<ServiceLocation<FixedOffset>, ParsingError> {
    if !location.has_tag_name("location") {
        return Err(ParsingError::InvalidTagName {
            expected: "location",
            found: location.tag_name().name().parse().unwrap()
        })
    }

    Ok(
        ServiceLocation {
            location: Location {
                name: location.field_text("locationName")?,
                crs: location.field_text("crs").ok(),
                tiploc: location.field_text("tiploc").ok()
            },
            associations: {
                match location.children().find(|x| x.has_tag_name("associations")) {
                    None => {None}
                    Some(associations) => {
                        let mut vec = Vec::new();

                        for node in associations.children() {
                            vec.push(parse_association(node)?)
                        }

                        Some(vec)
                    }
                }
            },
            adhoc_alerts: {
                match location.children().find(|x| x.has_tag_name("adhocAlerts")) {
                    None => {None}
                    Some(alerts) => {
                        todo!()
                    }
                }
            },
            activities: {
                match &*(location.field_text("activities")?) {
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
                match location.field::<u16>("length") {
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
            detach_front: location.field_bool("detachFront", false)?,
            operational: location.field_bool("isOperational", false)?,
            pass: location.field_bool("isPass", false)?,
            cancelled: location.field_bool("isCancelled", false)?,
            false_destination: {
                location.field_text("falseDest").ok().map(|name| Location {
                    name,
                    crs: None,
                    tiploc: location.field_text("fdTiploc").ok()
                })
            },
            platform: location.field::<u8>("platform").ok(),
            platform_hidden: location.field_bool("platformIsHidden", false)?,
            // The docs make this misspelling. Is it a mistake? Who knows!
            suppressed: location.field_bool("serviceIsSupressed", false)?,
            arrival_time: {
                match location.field_time("sta") {
                    Ok(sta) => {
                        let forecast_type = match &*location.field_text("arrivalType")? {
                            "Actual" => {ForecastType::Actual}
                            "Forecast" => {ForecastType::Estimated}
                            x => {return Err(ParsingError::InvalidForecast(x.parse().unwrap()))}
                        };

                        Some(
                            ServiceTime {
                                scheduled: Some(sta),
                                time: match forecast_type {
                                    ForecastType::Actual => {location.field_time("ata").ok()}
                                    ForecastType::Estimated => {location.field_time("eta").ok()}
                                },
                                forecast_type: Some(forecast_type),
                                source: location.field_text("arrivalSource").ok()
                            }
                        )
                    }
                    Err(_) => {None}
                }
            },
            departure_time: {
                match location.field_time("std") {
                    Ok(std) => {
                        let forecast_type = match &*location.field_text("departureType")? {
                            "Actual" => {ForecastType::Actual}
                            "Forecast" => {ForecastType::Estimated}
                            x => {return Err(ParsingError::InvalidForecast(x.parse().unwrap()))}
                        };

                        Some(
                            ServiceTime {
                                scheduled: Some(std),
                                time: match forecast_type {
                                    ForecastType::Actual => {location.field_time("atd").ok()}
                                    ForecastType::Estimated => {location.field_time("etd").ok()}
                                },
                                forecast_type: Some(forecast_type),
                                source: location.field_text("departureSource").ok()
                            }
                        )
                    }
                    Err(_) => {None}
                }
            },

            #[allow(deprecated)]
            lateness: location.field_text("lateness").ok()
        }
    )
}

fn parse_service_details(details: Node) -> Result<ServiceDetails<FixedOffset>, ParsingError> {
    if !details.has_tag_name("GetServiceDetailsResult") {
        return Err(ParsingError::InvalidTagName {
            expected: "GetServiceDetailsResult",
            found: details.tag_name().name().parse().unwrap()
        })
    }

    let typ = &*details.field_text("serviceType")?;

    if typ != "train" {
        return Err(ParsingError::UnsupportedServiceType(typ.parse().unwrap()))
    }

    Ok(
        ServiceDetails {
            generated_at: details.field_time("generatedAt")?,
            rid: details.field_text("rid")?,
            uid: details.field_text("uid")?,
            rsid: details.field_text("rsid").ok(),
            trainid: details.field_text("trainid")?,
            sdd: details.field_date("sdd")?,
            passenger_service: details.field_bool("isPassengerService", true)?,
            charter: details.field_bool("isCharter", false)?,
            category: details.field_text("category")?,
            operator: details.field_text("operator")?,
            operator_code: details.field_text("operatorCode")?,
            cancel_reason: details.field_text("cancelReason").ok(),
            delay_reason: details.field_text("delayReason").ok(),
            reverse_formation: details.field_bool("isReverseFormation", false)?,
            locations: {
                let mut vec = Vec::new();

                for node in details.children()
                    .find(|x| x.has_tag_name("locations"))
                    .ok_or(ParsingError::MissingField("locations"))?
                    .children() {
                    vec.push(parse_service_location(node)?)
                }

                vec
            }
        }
    )
}

/// A fetch error.
/// This describes an error that occurred while making a request to OpenLDBSVWS.
#[derive(Error, Debug)]
#[cfg(feature = "reqwest")]
pub enum FetchError {
    /// An error returned by the server.
    #[error("server responded with error {error:?}")]
    StatusError {
        error: u16,
        document: String
    },
    /// An error while sending the request.
    #[error("couldn't send request")]
    RequestError {
        source: reqwest::Error
    },
    /// An error while parsing the XML document into a struct.
    #[error("couldn't parse")]
    ParseError {
        source: ParsingError
    },
    /// An error while parsing the response into an XML document.
    #[error("malformed XML document")]
    ParseXMLError {
        source: roxmltree::Error
    }
}

#[cfg(feature = "reqwest")]
pub async fn get_arrival_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>, FetchError> {
    todo!()
}

#[cfg(feature = "reqwest")]
pub async fn get_departure_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>, FetchError> {
    todo!()
}

/// Gets the service details of a service given it's RTTI ID and a valid OpenLDBSVWS (not OpenLDBWS)
/// token.
#[cfg(feature = "reqwest")]
pub async fn get_service_details(client: Client, token: &str, rid: &str) -> Result<ServiceDetails<FixedOffset>, FetchError> {
    let service_details_payload = format!(service_details!(), token = token, rid = rid);
    let res = client.post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
        .body(service_details_payload)
        .timeout(Duration::new(5, 0))
        .header("Content-Type", "text/xml")
        .header("Accept", "text/xml")
        .send()
        .await
        .map_err(|e| FetchError::RequestError {
            source: e
        })?;

    let status = res.status();
    let result = res.text()
        .await
        .map_err(|e| FetchError::RequestError {
            source: e
        })?;

    if !status.is_success() {
        return Err(FetchError::StatusError {
            error: status.as_u16(),
            document: result
        })
    }

    let doc = Document::parse(&result)
        .map_err(|e| FetchError::ParseXMLError {
            source: e
        })?;

    let response = doc.root()
        .descendants()
        .find(|x| x.has_tag_name("GetServiceDetailsByRIDResponse"))
        .ok_or(ParsingError::MissingField("GetServiceDetailsByRIDResponse"))
        .map_err(|e| FetchError::ParseError {
            source: e
        })?;

    let details = response.children()
        .find(|x| x.has_tag_name("GetServiceDetailsResult"))
        .ok_or(ParsingError::MissingField("GetServiceDetailsResult"))
        .map_err(|e| FetchError::ParseError {
            source: e
        })?;

    parse_service_details(details)
        .map_err(|e| FetchError::ParseError {
            source: e
        })
}
