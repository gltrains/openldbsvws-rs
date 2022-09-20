use crate::parsable::ParsingError;
use chrono::{DateTime, FixedOffset, NaiveDate};
use core::any::type_name;
use core::str::FromStr;
use roxmltree::Children;

#[cfg(feature = "roxmltree")]
use roxmltree::Node;

pub trait Traversable: Sized {
    type Iter: Iterator<Item = Self> + IntoIterator<Item = Self>;

    fn child(&self, name: &'static str) -> Result<Self, ParsingError>;
    fn children(&self) -> Self::Iter;
    fn tag_name(&self) -> &str;
    fn get_text(&self) -> Result<String, ParsingError>;
    fn get<T: FromStr>(&self) -> Result<T, ParsingError>;
    fn get_time(&self) -> Result<DateTime<FixedOffset>, ParsingError>;
    fn get_date(&self) -> Result<NaiveDate, ParsingError>;
    fn get_bool(&self, default: bool) -> Result<bool, ParsingError>;
}

#[cfg(feature = "roxmltree")]
impl<'a, 'b> Traversable for Node<'a, 'b> {
    type Iter = Children<'a, 'b>;

    fn child(&self, name: &'static str) -> Result<Self, ParsingError> {
        self.children()
            .find(|x| x.has_tag_name(name))
            .ok_or(ParsingError::MissingField(name.to_owned()))
    }

    fn children(&self) -> Self::Iter {
        self.children()
    }

    fn tag_name(&self) -> &'a str {
        self.tag_name().name()
    }

    fn get_text(&self) -> Result<String, ParsingError> {
        self.text()
            .ok_or(ParsingError::InvalidField {
                field: Traversable::tag_name(self).to_owned(),
                expected: "text",
                found: None,
            })
            .map(|x| x.to_owned())
    }

    fn get<T: FromStr>(&self) -> Result<T, ParsingError> {
        let text = self.get_text()?;

        text.parse::<T>().map_err(|_| ParsingError::InvalidField {
            field: Traversable::tag_name(self).to_owned(),
            expected: type_name::<T>(),
            found: Some(text),
        })
    }

    fn get_time(&self) -> Result<DateTime<FixedOffset>, ParsingError> {
        let text = self.get_text()?;

        DateTime::parse_from_rfc3339(&text).map_err(|_| ParsingError::InvalidField {
            field: Traversable::tag_name(self).to_owned(),
            expected: "DateTime",
            found: Some(text),
        })
    }

    fn get_date(&self) -> Result<NaiveDate, ParsingError> {
        let text = self.get_text()?;

        NaiveDate::parse_from_str(&text, "%Y-%m-%d").map_err(|_| ParsingError::InvalidField {
            field: Traversable::tag_name(self).to_owned(),
            expected: "NaiveDate",
            found: Some(text),
        })
    }

    fn get_bool(&self, default: bool) -> Result<bool, ParsingError> {
        match self.get_text() {
            Ok(x) => match &*x {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ParsingError::InvalidField {
                    field: Traversable::tag_name(self).to_owned(),
                    expected: "bool",
                    found: Some(x),
                }),
            },
            Err(_) => Ok(default),
        }
    }
}
