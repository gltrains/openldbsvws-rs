use chrono::NaiveDate;
use roxmltree::Node;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parsable::{Parsable, ParsingError};
use crate::services::Location;
use crate::{bool, date, name, text};

/// Train association categories.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum AssociationCategory {
    /// A train joins this train.
    Join,
    /// A train divides from this train.
    Divide,
    /// Next.
    Next,
}

/// A train association.
///
/// A train can join, divide, link from and link to another train.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Association<'a> {
    /// The association category.
    pub category: AssociationCategory,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    pub rid: &'a str,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI allocated replacement.
    pub uid: &'a str,
    /// The Train ID value (headcode) for this service.
    pub trainid: &'a str,
    /// The Retail Service ID for this service, if known.
    pub rsid: Option<&'a str>,
    /// The Scheduled Departure Date of this service.
    pub sdd: NaiveDate,
    /// The origin location of the associated service.
    pub origin: Option<Location<'a>>,
    /// The destination location of the associated service.
    pub destination: Option<Location<'a>>,
    /// If true, this association is cancelled and will no longer happen.
    pub cancelled: bool,
}

impl<'a, 'b> Parsable<'a, 'a, 'b> for Association<'b> {
    fn parse(association: &Node<'a, 'a>, string: &'b str) -> Result<Self, ParsingError<'b>> {
        if name!(association) != "association" {
            return Err(ParsingError::InvalidTagName("association"));
        }

        Ok(Association {
            category: match text!(string, association, "category")? {
                "divide" => AssociationCategory::Divide,
                "join" => AssociationCategory::Join,
                "next" => AssociationCategory::Next,
                x => return Err(ParsingError::InvalidAssociationCategory(x)),
            },
            rid: text!(string, association, "rid")?,
            uid: text!(string, association, "uid")?,
            trainid: text!(string, association, "trainid")?,
            rsid: text!(string, association, "rsid").ok(),
            sdd: date!(string, association, "sdd")?,
            origin: Some(Location {
                name: text!(string, association, "origin")?,
                crs: text!(string, association, "originCRS").ok(),
                tiploc: text!(string, association, "originTiploc").ok(),
            }),
            destination: Some(Location {
                name: text!(string, association, "destination")?,
                crs: text!(string, association, "destCRS").ok(),
                tiploc: text!(string, association, "destTiploc").ok(),
            }),
            cancelled: bool!(string, association, "cancelled", false)?,
        })
    }
}
