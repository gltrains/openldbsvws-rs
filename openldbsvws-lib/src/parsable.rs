use crate::traversable::Traversable;
use thiserror::Error;

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
        found: String,
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
    MissingField(String),
    /// An invalid field.
    #[error("field {field:?} couldn't be parsed, expected {expected:?}, got {found:?}")]
    InvalidField {
        /// The field name.
        field: String,
        /// What was expected. This is purely for diagnostic reasons.
        expected: &'static str,
        /// The contents of the field.
        found: Option<String>,
    },
    /// An unsupported service type. The string represents the service type that was found.
    #[error("unsupported service type {0}")]
    UnsupportedServiceType(String),
}

pub trait Parsable: Sized {
    fn parse(from: impl Traversable) -> Result<Self, ParsingError>;
}
