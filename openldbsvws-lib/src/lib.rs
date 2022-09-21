#![feature(generic_associated_types)]

mod associations;
mod location;
mod parsable;
mod services;

pub use services::ServiceDetails;

#[cfg(feature = "reqwest")]
use reqwest::Client;

#[cfg(feature = "roxmltree")]
use roxmltree::{Document, Node};

// Why are these macros and not consts?
// For some reason, format! does not support
// consts.

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
pub async fn get_service_details<'a>(
    client: Client,
    token: &str,
    rid: &str,
) -> Result<ServiceDetails<'a>, FetchError<'a>> {

}


 */
