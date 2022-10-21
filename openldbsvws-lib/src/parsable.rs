use roxmltree::Node;
use chrono::{DateTime, FixedOffset, NaiveDate};
use thiserror::Error;
use std::error;

/// A parsing error.
/// This describes an error that occurred while translating XML into a struct.
#[derive(Error, Debug)]
pub enum ParsingError<'a> {
    /// An invalid tag name.
    #[error("invalid tag name, expected {0}")]
    InvalidTagName(&'a str),
    /// An invalid activity. The string represents the activity that was found.
    #[error("invalid activity, got {0}")]
    InvalidActivity(&'a str),
    /// An invalid forecast type. The string represents the forecast type that was found.
    #[error("invalid forecast type, got {0}")]
    InvalidForecast(&'a str),
    /// An invalid association category type. The string represents the category that was found.
    #[error("invalid association category, expected Join or Divide, got {0}")]
    InvalidAssociationCategory(&'a str),
    /// A missing field. The string represents the field that was not found.
    #[error("field {0} is missing")]
    MissingField(&'a str),
    /// An invalid field.
    #[error("field {field:?} couldn't be parsed, expected {expected:?}, got {found:?}")]
    InvalidField {
        /// The field name.
        field: &'a str,
        /// What was expected. This is purely for diagnostic reasons.
        expected: &'a str,
        /// The contents of the field.
        found: Option<&'a str>
    },
    /// An unsupported service type. The string represents the service type that was found.
    #[error("unsupported service type {0}")]
    UnsupportedServiceType(&'a str),
    /// XML parsing error.
    #[error("cannot parse XML")]
    XMLParseError { source: roxmltree::Error },
}

type ParsingResult<'a, T> = Result<T, ParsingError<'a>>;

pub(crate) trait Traversable<'a> {
    fn child(&self, name: &'static str) -> ParsingResult<Self> where Self: Sized;
    fn name(&self) -> &'a str;
    fn text(&self) -> ParsingResult<&'a str>;
    fn time(&self) -> ParsingResult<DateTime<FixedOffset>>;
    fn date(&self) -> ParsingResult<NaiveDate>;
    fn bool(&self, default: bool) -> bool;
    fn parse<T>(&self) -> ParsingResult<T>;
}

impl<'a> Traversable<'a> for Node<'a, '_> {
    fn child(&self, name: &'static str) -> ParsingResult<Self> {
        self.children()
            .find(|x| x.has_tag_name(name))
            .ok_or(ParsingError::MissingField(name))
    }

    fn name(&self) -> &'a str {
        self.tag_name().name()
    }

    fn text(&self) -> ParsingResult<&'a str> {
        self.first_child()
            .map(|text| {
                if !text.is_text() {
                    return ""
                }

                let range = text.range();
                let original = self.document().input_text();

                &original[range.start..range.end]
            })
            .ok_or_else(|| ParsingError::MissingField(self.name()))
    }

    fn time(&self) -> ParsingResult<DateTime<FixedOffset>> {
        let text = Traversable::text(self)?;

        DateTime::parse_from_rfc3339(text).map_err(|err| ParsingError::InvalidField {
            field: self.name(),
            expected: "DateTime",
            found: Some(text)
        })
    }

    fn date(&self) -> ParsingResult<NaiveDate> {
        let text = Traversable::text(self)?;

        NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|err| ParsingError::InvalidField {
            field: self.name(),
            expected: "NaiveDate",
            found: Some(text)
        })
    }

    fn bool(&self, default: bool) -> bool {
        todo!()
    }

    fn parse<T>(&self) -> ParsingResult<T> {
        let text = self.text();

        todo!()
    }
}

pub trait Parsable<'a>: Sized {
    fn parse(from: &'a Node<'a, '_>) -> Result<Self, ParsingError<'a>>;
}
