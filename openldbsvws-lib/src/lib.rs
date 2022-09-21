#![feature(generic_associated_types)]

mod associations;
mod location;
mod parsable;
mod services;

use associations::{Association, AssociationCategory};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use core::any::type_name;
use core::result::Result;
use core::str::from_utf8;
use core::str::FromStr;
use core::time::Duration;
use parsable::{Parsable, ParsingError};
use services::{Activity, ForecastType, Location, ServiceDetails, ServiceLocation, ServiceTime};
use std::borrow::Borrow;
use thiserror::Error;

#[cfg(feature = "reqwest")]
use reqwest::Client;

#[cfg(feature = "roxmltree")]
use roxmltree::{Document, Node};

// Why are these macros and not consts?
// For some reason, format! does not support
// consts.

macro_rules! service_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldbsv/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetServiceDetailsByRIDRequest><ldb:rid>{rid}</ldb:rid></ldb:GetServiceDetailsByRIDRequest></soapenv:Body></soapenv:Envelope>"}
}

macro_rules! arrival_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldb/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetArrivalBoardRequest><ldb:numRows>150</ldb:numRows><ldb:crs>{crs}</ldb:crs><ldb:filterCrs>{filter_crs}</ldb:filterCrs><ldb:filterType>{filter_type}</ldb:filterType><ldb:timeOffset>{time_offset}</ldb:timeOffset><ldb:timeWindow>{time_window}</ldb:timeWindow></ldb:GetArrivalBoardRequest></soapenv:Body></soapenv:Envelope>"}
}

/*

/// A fetch error.
/// This describes an error that occurred while making a request to OpenLDBSVWS.
#[derive(Error, Debug)]
#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub enum FetchError<'a> {
    /// An error returned by the server.
    #[error("server responded with error {error:?}")]
    StatusError { error: u16 },
    /// An error while sending the request.
    #[error("couldn't send request")]
    RequestError { source: reqwest::Error },
    /// An error while parsing the XML document into a struct.
    #[error("couldn't parse")]
    ParseError { source: ParsingError<'a> },
    /// An error while parsing the response into an XML document.
    #[error("malformed XML document")]
    ParseXMLError { source: roxmltree::Error },
}

#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub async fn get_arrival_details<'a>(
    client: Client,
    token: &str,
    station: &str,
) -> Result<Document<'a>, FetchError<'a>> {
    todo!()
}

#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub async fn get_departure_details<'a>(
    client: Client,
    token: &str,
    station: &str,
) -> Result<Document<'a>, FetchError<'a>> {
    todo!()
}



/// Gets the service details of a service given it's RTTI ID and a valid OpenLDBSVWS (not OpenLDBWS)
/// token.
#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub async fn get_service_details<'a>(client: Client, token: &str, rid: &str) -> ServiceDetails<'a> {
    let service_details_payload = format!(service_details!(), token = token, rid = rid);
    let res = client
        .post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
        .body(service_details_payload)
        .timeout(Duration::new(5, 0))
        .header("Content-Type", "text/xml")
        .header("Accept", "text/xml")
        .send()
        .await
        .unwrap();

    let status = res.status();

    let document = res.text().await.unwrap();

    if !status.is_success() {
        panic!();
    }

    ServiceDetails::try_from(&*document).unwrap()
}


 */
