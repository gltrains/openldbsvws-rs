use crate::parsable::Parsable;
use crate::services::Location;
use crate::{ParsingError, Traversable};
use chrono::NaiveDate;

/// Train association categories.
#[derive(Debug)]
pub enum AssociationCategory {
    /// A train joins this train.
    Join,
    /// A train divides from this train.
    Divide,
    /// A train links from this train.
    LinkedFrom,
    /// A train links to this train.
    LinkedTo,
}

/// A train association.
///
/// A train can join, divide, link from and link to another train.
#[derive(Debug)]
pub struct Association {
    /// The association category.
    pub category: AssociationCategory,
    /// A unique RTTI ID for this service that can be used to obtain full details of the service.
    pub rid: String,
    /// The TSDB Train UID value for this service, or if one is not available, then an RTTI allocated replacement.
    pub uid: String,
    /// The Train ID value (headcode) for this service.
    pub trainid: String,
    /// The Retail Service ID for this service, if known.
    pub rsid: Option<String>,
    /// The Scheduled Departure Date of this service.
    pub sdd: NaiveDate,
    /// The origin location of the associated service.
    pub origin: Option<Location>,
    /// The destination location of the associated service.
    pub destination: Option<Location>,
    /// If true, this association is cancelled and will no longer happen.
    pub cancelled: bool,
}

impl Parsable for Association {
    fn parse(association: impl Traversable) -> Result<Self, ParsingError> {
        if association.tag_name() != "association" {
            return Err(ParsingError::InvalidTagName {
                expected: "association",
                found: association.tag_name().parse().unwrap(),
            });
        }

        Ok(Association {
            category: match &*association.child("category")?.get_text()? {
                "divide" => AssociationCategory::Divide,
                "join" => AssociationCategory::Join,
                x => return Err(ParsingError::InvalidAssociationCategory(x.parse().unwrap())),
            },
            rid: association.child("rid")?.get_text()?,
            uid: association.child("uid")?.get_text()?,
            trainid: association.child("trainid")?.get_text()?,
            rsid: association.child("rsid")?.get_text().ok(),
            sdd: association.child("sdd")?.get_date()?,
            origin: Some(Location {
                name: association.child("origin")?.get_text()?,
                crs: association.child("originCRS")?.get_text().ok(),
                tiploc: association.child("originTiploc")?.get_text().ok(),
            }),
            destination: Some(Location {
                name: association.child("destination")?.get_text()?,
                crs: association.child("destCRS")?.get_text().ok(),
                tiploc: association.child("destTiploc")?.get_text().ok(),
            }),
            cancelled: association.child("cancelled")?.get_bool(false)?,
        })
    }
}
