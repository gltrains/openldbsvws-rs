use roxmltree::{Document, Node};
use reqwest::Client;
use std::time::Duration;
use std::str::FromStr;
use chrono::{TimeZone, DateTime, FixedOffset, NaiveDate};
use anyhow::{anyhow, Context, Result, Error};

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
    destination_crs: Option<String>,
    destination_tiploc: Option<String>
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
pub struct Association<T: TimeZone> {
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
    sdd: DateTime<T>,
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
#[derive(Debug)]
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
    PassesAnotherTrain
}

#[derive(Debug)]
/// A location in this service's schedule. Not all locations are stopped at.
pub struct ServiceLocation<T: TimeZone> {
    /// The location of this stop.
    location: Location,
    /// Associations that happen at this stop.
    associations: Option<Vec<Association<T>>>,
    /// Ad-hoc alerts about this stop.
    adhoc_alerts: Option<Vec<String>>,
    /// Activities that happen at this stop.
    activity: Option<Activity>,
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

#[inline(always)]
fn get_field_text(node: Node, name: &str) -> Result<String> {
    Ok(
        node.children()
            .find(|x| x.has_tag_name(name))
            .with_context(|| format!("couldn't find field {}", name))?
            .text()
            .with_context(|| format!("couldn't get {} text", name))?
            .to_string()
    )
}

#[inline(always)]
fn get_field<T: FromStr>(node: Node, name: &str) -> Result<T> {
    match get_field_text(node, name)?.parse::<T>() {
        Ok(x) => {Ok(x)}
        Err(_) => {Err(anyhow!("couldn't parse {}", name))}
    }
}

#[inline(always)]
fn get_field_time(node: Node, name: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(
        &*get_field_text(node, name)?
    ).with_context(|| format!("couldn't parse {} time", name))
}

#[inline(always)]
fn get_field_date(node: Node, name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(
        &*get_field_text(node, name)?,
        "%Y-%m-%d"
    ).with_context(|| format!("couldn't parse {} date", name))
}

#[inline(always)]
fn get_field_bool(node: Node, name: &str, default: bool) -> Result<bool> {
    match get_field_text(node, name) {
        Ok(x) => {
            match &*x {
                "true" => {
                    Ok(true)
                },
                "false" => {
                    Ok(false)
                },
                res => {
                    Err(anyhow!("field {} was found but was not true or false (got {})", name, res))
                }
            }
        }
        Err(_) => {Ok(default)}
    }
}

fn parse_association(association: Node) -> Result<Association<FixedOffset>> {
    if !association.has_tag_name("association") {
        return Err(anyhow!("not an association"))
    }

    todo!()
}

fn parse_service_location(location: Node) -> Result<ServiceLocation<FixedOffset>> {
    if !location.has_tag_name("location") {
        return Err(anyhow!("not a location, got a {}", location.tag_name().name()))
    }

    Ok(
        ServiceLocation {
            location: Location {
                name: get_field_text(location, "locationName")?,
                destination_crs: get_field_text(location, "crs").ok(),
                destination_tiploc: get_field_text(location, "tiploc").ok()
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
            activity: {
                match get_field_text(location, "activities")?.trim() {
                    "" => {None}
                    activity => {
                        Some(
                            match activity {
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

                                _ => {return Err(anyhow!("invalid activity code {}", activity))}
                            }
                        )
                    }
                }
            },
            length: {
                match get_field::<u16>(location, "length") {
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
            detach_front: get_field_bool(location, "detachFront", false)?,
            operational: get_field_bool(location, "isOperational", false)?,
            pass: get_field_bool(location, "isPass", false)?,
            cancelled: get_field_bool(location, "isCancelled", false)?,
            false_destination: {
                match get_field_text(location, "falseDest").ok() {
                    Some(name) => {
                        Some(
                            Location {
                                name,
                                destination_crs: None,
                                destination_tiploc: get_field_text(location, "fdTiploc").ok()
                            }
                        )
                    }
                    None => {None}
                }
            },
            platform: get_field::<u8>(location, "platform").ok(),
            platform_hidden: get_field_bool(location, "platformIsHidden", false)?,
            // The docs make this misspelling. Is it a mistake? Who knows!
            suppressed: get_field_bool(location, "serviceIsSupressed", false)?,
            arrival_time: {
                match get_field_time(location, "sta") {
                    Ok(sta) => {
                        let forecast_type = match &*get_field_text(location, "arrivalType")? {
                            "Actual" => {ForecastType::Actual}
                            "Forecast" => {ForecastType::Estimated}
                            x => {return Err(anyhow!("expected actual/forecast for arrivalType, got {}", x))}
                        };

                        Some(
                            ServiceTime {
                                scheduled: Some(sta),
                                time: match forecast_type {
                                    ForecastType::Actual => {get_field_time(location, "ata").ok()}
                                    ForecastType::Estimated => {get_field_time(location, "eta").ok()}
                                },
                                forecast_type: Some(forecast_type),
                                source: get_field_text(location, "arrivalSource").ok()
                            }
                        )
                    }
                    Err(_) => {None}
                }
            },
            departure_time: {
                match get_field_time(location, "std") {
                    Ok(std) => {
                        let forecast_type = match &*get_field_text(location, "departureType")? {
                            "Actual" => {ForecastType::Actual}
                            "Forecast" => {ForecastType::Estimated}
                            x => {return Err(anyhow!("expected actual/forecast for departureType, got {}", x))}
                        };

                        Some(
                            ServiceTime {
                                scheduled: Some(std),
                                time: match forecast_type {
                                    ForecastType::Actual => {get_field_time(location, "atd").ok()}
                                    ForecastType::Estimated => {get_field_time(location, "etd").ok()}
                                },
                                forecast_type: Some(forecast_type),
                                source: get_field_text(location, "departureSource").ok()
                            }
                        )
                    }
                    Err(_) => {None}
                }
            },
            lateness: get_field_text(location, "lateness").ok()
        }
    )
}

fn parse_service_details(details: Node) -> Result<ServiceDetails<FixedOffset>> {
    if !details.has_tag_name("GetServiceDetailsResult") {
        return Err(anyhow!("not a GetServiceDetailsResult"))
    }

    let typ = &*get_field_text(details, "serviceType")?;

    if typ != "train" {
        return Err(anyhow!("not a train service, is a {}", typ))
    }

    Ok(
        ServiceDetails {
            generated_at: get_field_time(details, "generatedAt")?,
            rid: get_field_text(details, "rid")?,
            uid: get_field_text(details, "uid")?,
            rsid: get_field_text(details, "rsid").ok(),
            trainid: get_field_text(details, "trainid")?,
            sdd: get_field_date(details, "sdd")?,
            passenger_service: get_field_bool(details, "isPassengerService", true)?,
            charter: get_field_bool(details, "isCharter", false)?,
            category: get_field_text(details, "category")?,
            operator: get_field_text(details, "operator")?,
            operator_code: get_field_text(details, "operatorCode")?,
            cancel_reason: get_field_text(details, "cancelReason").ok(),
            delay_reason: get_field_text(details, "delayReason").ok(),
            reverse_formation: get_field_bool(details, "isReverseFormation", false)?,
            locations: {
                let mut vec = Vec::new();

                for node in details.children().find(|x| x.has_tag_name("locations")).context("no locations")?.children() {
                    vec.push(parse_service_location(node)?)
                }

                vec
            }
        }
    )
}

pub async fn get_arrival_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>> {
    todo!()
}

pub async fn get_departure_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>> {
    todo!()
}

/// Gets the service details of a service given it's RTTI ID and a valid OpenLDBSVWS (not OpenLDBWS)
/// token.
pub async fn get_service_details(client: Client, token: &str, rid: &str) -> Result<ServiceDetails<FixedOffset>> {
    let service_details_payload = format!(service_details!(), token = token, rid = rid);
    let res = client.post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
        .body(service_details_payload)
        .timeout(Duration::new(5, 0))
        .header("Content-Type", "text/xml")
        .header("Accept", "text/xml")
        .send()
        .await
        .context("failed to send request")?;

    let result = res.text().await.context("couldn't get response result")?;

    let doc = Document::parse(&*result).context("couldn't parse document")?;

    let details = doc.root()
        .descendants()
        .find(|x| x.has_tag_name("GetServiceDetailsResult"))
        .context("server didn't return a result")?;

    parse_service_details(details)
}
