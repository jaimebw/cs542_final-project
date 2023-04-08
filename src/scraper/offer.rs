use crate::scraper::price::PriceUSD;
use select::node::Node;
use select::predicate::{And, Attr, Class, Name, Text};
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug)]
pub struct Offer {
    pub condition: Condition,
    pub condition_description: Option<String>,
    pub price: PriceUSD,
    pub ships_from: String,
    pub sold_by: String,
    /// Seller page may be None since the seller "Amazon.com" does not have a seller page.
    pub seller_page: Option<String>,
}

#[derive(Debug)]
pub struct MissingOfferField(&'static str);

impl<'a> TryFrom<Node<'a>> for Offer {
    type Error = MissingOfferField;

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        let price = value
            .find(Class("a-price"))
            .flat_map(|node| node.find(Class("a-offscreen")))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .filter_map(|text| PriceUSD::from_str(text).ok())
            .next()
            .ok_or(MissingOfferField("price"))?;

        let condition = value
            .find(Attr("id", "aod-offer-heading"))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .filter_map(|text| Condition::from_str(text).ok())
            .next()
            .ok_or(MissingOfferField("condition"))?;

        let condition_description = value
            .find(Attr("id", "aod-condition-container"))
            .flat_map(|node| node.find(Class("expandable-expanded-text")))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .next()
            .map(str::to_string);

        let ships_from = value
            .find(Attr("id", "aod-offer-shipsFrom"))
            .flat_map(|node| node.find(Class("a-col-right")))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .map(|text| text.trim())
            .collect::<String>();

        if ships_from.is_empty() {
            return Err(MissingOfferField("ships_from"));
        }

        let seller = value
            .find(Attr("id", "aod-offer-soldBy"))
            .flat_map(|node| node.find(Class("a-col-right")))
            .next()
            .ok_or(MissingOfferField("sold_by"))?;

        let (sold_by, seller_page) = match seller.find(Name("a")).next() {
            Some(link) => {
                let seller_name = link
                    .find(Text)
                    .filter_map(|node| node.as_text())
                    .map(|text| text.trim())
                    .collect();

                (seller_name, link.attr("href").map(str::to_string))
            }
            None => {
                // As far as I know, this is only done when the seller is "Amazon.com"
                let seller_name = seller
                    .find(And(
                        Name("span"),
                        Attr("class", "a-size-small a-color-base"),
                    ))
                    .flat_map(|node| node.find(Text))
                    .filter_map(|node| node.as_text())
                    .map(|text| text.trim())
                    .collect::<String>();

                (seller_name, None)
            }
        };

        Ok(Offer {
            condition,
            condition_description,
            price,
            ships_from,
            sold_by,
            seller_page,
        })
    }
}

/// https://www.amazon.com/gp/help/customer/display.html?nodeId=202074290
#[derive(Debug, Eq, PartialEq)]
pub enum Condition {
    New,
    Renewed,
    UsedLikeNew,
    UsedVeryGood,
    UsedGood,
    UsedAcceptable,
}

#[derive(Copy, Clone, Debug)]
pub struct UnknownCondition;

impl FromStr for Condition {
    type Err = UnknownCondition;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();

        if s == "New" {
            return Ok(Condition::New);
        }

        if let Some(suffix) = s.strip_prefix("Used") {
            // Trimming is performed here instead of using a single large match statement because
            // there is a large amount of whitespace after "Used".
            return match suffix.trim_start() {
                "- Renewed" => Ok(Condition::Renewed),
                "- Like New" => Ok(Condition::UsedLikeNew),
                "- Very Good" => Ok(Condition::UsedVeryGood),
                "- Good" => Ok(Condition::UsedGood),
                "- Acceptable" => Ok(Condition::UsedAcceptable),
                _ => Err(UnknownCondition),
            };
        }

        Err(UnknownCondition)
    }
}
