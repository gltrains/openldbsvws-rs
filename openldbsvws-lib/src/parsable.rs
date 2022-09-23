use roxmltree::Node;
use thiserror::Error;

/// A parsing error.
/// This describes an error that occurred while translating XML into a struct.
#[derive(Error, Debug)]
pub enum ParsingError<'a> {
    /// An invalid tag name.
    #[error("invalid tag name, expected {0}")]
    InvalidTagName(&'static str),
    /// An invalid activity. The string represents the activity that was found.
    #[error("invalid activity, got {0}")]
    InvalidActivity(&'a str),
    /// An invalid forecast type. The string represents the forecast type that was found.
    #[error("invalid forecast type, expected Actual or Forecast, got {0}")]
    InvalidForecast(&'a str),
    /// An invalid association category type. The string represents the category that was found.
    #[error("invalid association category, expected Join or Divide, got {0}")]
    InvalidAssociationCategory(&'a str),
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
        found: Option<&'a str>,
    },
    /// An unsupported service type. The string represents the service type that was found.
    #[error("unsupported service type {0}")]
    UnsupportedServiceType(&'a str),
    /// XML parsing error.
    #[error("cannot parse XML")]
    XMLParseError { source: roxmltree::Error },
}

#[macro_export]
macro_rules! child {
    ($x: expr, $y: literal) => {
        $x.children()
            .find(|x| x.has_tag_name($y))
            .ok_or(ParsingError::MissingField($y))
    };
}

#[macro_export]
macro_rules! name {
    ($x: expr) => {
        $x.tag_name().name()
    };
}

#[macro_export]
macro_rules! text {
    ($t: expr, $x: expr, $y: literal) => {
        $x.children()
            .find(|x| x.has_tag_name($y))
            .ok_or(ParsingError::MissingField($y))
            .map(|x| {
                // Oh roxmltree how I hate you oh so much
                // First get the text node:
                if let Some(text) = x.first_child() {
                    if !text.is_text() {
                        return "";
                    }

                    // Then get the range:
                    let range = text.range();

                    // Then map that range to the original string:
                    &$t[range.start..range.end]
                } else {
                    ""
                }
            })
    };
}

#[macro_export]
macro_rules! time {
    ($t: expr, $x: expr, $y: literal) => {
        match text!($t, $x, $y) {
            Ok(text) => {
                DateTime::parse_from_rfc3339(text).map_err(|_| ParsingError::InvalidField {
                    field: $y,
                    expected: "DateTime",
                    found: Some(text),
                })
            }
            Err(e) => Err(e),
        }
    };
}

#[macro_export]
macro_rules! date {
    ($t: expr, $x: expr, $y: literal) => {{
        let text = text!($t, $x, $y)?;

        NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| ParsingError::InvalidField {
            field: $y,
            expected: "NaiveDate",
            found: Some(text),
        })
    }};
}

#[macro_export]
macro_rules! bool {
    ($t: expr, $x: expr, $y: literal, $z: literal) => {
        match text!($t, $x, $y) {
            Ok(x) => match x {
                "true" => Ok(true),
                "false" => Ok(false),
                "" => Ok($z),
                x => Err(ParsingError::InvalidField {
                    field: $y,
                    expected: "bool",
                    found: Some(x),
                }),
            },
            Err(_) => Ok($z),
        }
    };
}

#[macro_export]
macro_rules! parse {
    ($t: expr, $x: expr, $y: literal, $z: ty) => {
        text!($t, $x, $y)
            .unwrap_or("")
            .parse::<$z>()
    }
}

pub trait Parsable<'a, 'b, 'c>: Sized {
    fn parse(from: &Node<'a, 'b>, string: &'c str) -> Result<Self, ParsingError<'c>>;
}
