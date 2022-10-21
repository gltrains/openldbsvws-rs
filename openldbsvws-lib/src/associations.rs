use chrono::NaiveDate;
use roxmltree::Node;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parsable::{Parsable, ParsingError, Traversable};
use crate::services::Location;

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

impl<'a> Parsable<'a> for Association<'a> {
    fn parse(association: &'a Node<'a, '_>) -> Result<Self, ParsingError<'a>> {
        if association.name() != "association" {
            return Err(ParsingError::InvalidTagName("association"));
        }

        Ok(Association {
            category: match Traversable::text(&association.child("category")?)? {
                "divide" => AssociationCategory::Divide,
                "join" => AssociationCategory::Join,
                "next" => AssociationCategory::Next,
                x => return Err(ParsingError::InvalidAssociationCategory(x)),
            },
            rid: Traversable::text(&association.child("rid")?)?,
            uid: Traversable::text(&association.child("uid")?)?,
            trainid: Traversable::text(&association.child("trainid")?)?,
            rsid: Traversable::text(&association.child("rsid")?).ok(),
            sdd: association.child("sdd")?.date()?,
            origin: Some(Location {
                name: Traversable::text(&association.child("origin")?)?,
                crs: Traversable::text(&association.child("originCRS")?).ok(),
                tiploc: Traversable::text(&association.child("originTiploc")?).ok(),
            }),
            destination: Some(Location {
                name: Traversable::text(&association.child("destination")?)?,
                crs: Traversable::text(&association.child("destCRS")?).ok(),
                tiploc: Traversable::text(&association.child("destTiploc")?).ok(),
            }),
            cancelled: association.child("cancelled")?.bool(false),
        })
    }
}
