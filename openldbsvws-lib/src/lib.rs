use roxmltree::Document;
use reqwest::Client;
use std::time::Duration;
use std::str;
use chrono::{TimeZone, DateTime};
use anyhow::{anyhow, Context, Result};

// Why are these macros and not consts?
// For some reason, format! does not support
// consts.

macro_rules! service_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldbsv/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetServiceDetailsByRIDRequest><ldb:rid>{rid}</ldb:rid></ldb:GetServiceDetailsByRIDRequest></soapenv:Body></soapenv:Envelope>"}
}

macro_rules! arrival_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldb/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetArrivalBoardRequest><ldb:numRows>150</ldb:numRows><ldb:crs>{crs}</ldb:crs><ldb:filterCrs>{filter_crs}</ldb:filterCrs><ldb:filterType>{filter_type}</ldb:filterType><ldb:timeOffset>{time_offset}</ldb:timeOffset><ldb:timeWindow>{time_window}</ldb:timeWindow></ldb:GetArrivalBoardRequest></soapenv:Body></soapenv:Envelope>"}
}

pub struct Location {
    name: String,
    destination_crs: Option<String>,
    destination_tiploc: String
}

pub enum AssociationCategory {
    Join,
    Divide,
    LinkedFrom,
    LinkedTo
}

pub struct Association<T: TimeZone> {
    category: AssociationCategory,
    rid: String,
    uid: String,
    trainid: String,
    rsid: String,
    sdd: DateTime<T>,
    origin: Option<Location>,
    destination: Option<Location>,
    cancelled: bool
}

pub enum ForecastType {
    Estimated,
    Actual,
    Unknown
}

pub struct ServiceTime<T: TimeZone> {
    scheduled: DateTime<T>,
    time: Option<DateTime<T>>,
    forecast_type: Option<ForecastType>,
    source: Option<String>
}

pub struct ServiceLocation<T: TimeZone> {
    location: Location,
    associations: Vec<Association<T>>,
    adhoc_alerts: Vec<String>,
    activities: Vec<String>,
    length: u16,
    detach_front: bool, // someone fucked up the docs for this
    operational: bool,
    pass: bool,
    cancelled: bool,
    false_destination: Option<Location>,
    platform: u8,
    platform_hidden: bool,
    suppressed: bool,
    arrival_time: Option<ServiceTime<T>>,
    departure_time: Option<ServiceTime<T>>,

    #[deprecated(note = "lateness is not guaranteed to be parseable to an int, please use arrival and departure time")]
    lateness: String
}

pub struct ServiceDetails<T: TimeZone> {
    generated_at: DateTime<T>,
    rid: String,
    uid: String,
    rsid: String,
    trainid: String,
    sdd: DateTime<T>,
    passenger_service: bool,
    charter: bool,
    category: String,
    operator: String,
    operator_code: String,
    cancel_reason: String,
    delay_reason: String,
    reverse_formation: bool,
    locations: Vec<ServiceLocation<T>>
}

pub async fn get_arrival_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>> {
    todo!()
}

pub async fn get_departure_details<'a>(client: Client, token: &str, station: &str) -> Result<Document<'a>> {
    todo!()
}

pub async fn get_service_details<'a>(client: Client, token: &str, rid: &str) -> Result<Document<'a>> {
    let service_details_payload = format!(service_details!(), token=token, rid=rid);
    let res = client.post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
        .body(service_details_payload)
        .timeout(Duration::new(5, 0))
        .header("Content-Type", "text/xml")
        .header("Accept", "text/xml")
        .send()
        .await
        .context("failed to send request")?;

    let result = res.text().await.context("couldn't get response result")?;
    
    let doc = Document::parse(Box::leak(Box::new(result))).context("couldn't parse document");

    todo!();
}
