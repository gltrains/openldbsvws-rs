#![feature(generic_associated_types)]

mod associations;
mod location;
mod parsable;
mod services;
mod traversable;

use associations::{Association, AssociationCategory};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use core::any::type_name;
use core::result::Result;
use core::str::from_utf8;
use core::str::FromStr;
use core::time::Duration;
use parsable::{Parsable, ParsingError};
use services::{Activity, ForecastType, Location, ServiceDetails, ServiceLocation, ServiceTime};
use thiserror::Error;
use traversable::Traversable;

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

/// A fetch error.
/// This describes an error that occurred while making a request to OpenLDBSVWS.
#[derive(Error, Debug)]
#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub enum FetchError {
    /// An error returned by the server.
    #[error("server responded with error {error:?}")]
    StatusError { error: u16, document: String },
    /// An error while sending the request.
    #[error("couldn't send request")]
    RequestError { source: reqwest::Error },
    /// An error while parsing the XML document into a struct.
    #[error("couldn't parse")]
    ParseError { source: ParsingError },
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
) -> Result<Document<'a>, FetchError> {
    todo!()
}

#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub async fn get_departure_details<'a>(
    client: Client,
    token: &str,
    station: &str,
) -> Result<Document<'a>, FetchError> {
    todo!()
}

/// Gets the service details of a service given it's RTTI ID and a valid OpenLDBSVWS (not OpenLDBWS)
/// token.
#[cfg(feature = "reqwest")]
#[cfg(feature = "roxmltree")]
pub async fn get_service_details(
    client: Client,
    token: &str,
    rid: &str,
) -> Result<ServiceDetails, FetchError> {
    let service_details_payload = format!(service_details!(), token = token, rid = rid);
    let res = client
        .post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
        .body(service_details_payload)
        .timeout(Duration::new(5, 0))
        .header("Content-Type", "text/xml")
        .header("Accept", "text/xml")
        .send()
        .await
        .map_err(|e| FetchError::RequestError { source: e })?;

    let status = res.status();
    let result = res
        .text()
        .await
        .map_err(|e| FetchError::RequestError { source: e })?;

    if !status.is_success() {
        return Err(FetchError::StatusError {
            error: status.as_u16(),
            document: result,
        });
    }

    let doc = Document::parse(&result).map_err(|e| FetchError::ParseXMLError { source: e })?;

    let response = doc
        .root()
        .descendants()
        .find(|x| x.has_tag_name("GetServiceDetailsByRIDResponse"))
        .ok_or(ParsingError::MissingField("GetServiceDetailsByRIDResponse"))
        .map_err(|e| FetchError::ParseError { source: e })?;

    let details = response
        .children()
        .find(|x| x.has_tag_name("GetServiceDetailsResult"))
        .ok_or(ParsingError::MissingField("GetServiceDetailsResult"))
        .map_err(|e| FetchError::ParseError { source: e })?;

    ServiceDetails::parse(details).map_err(|e| FetchError::ParseError { source: e })
}
