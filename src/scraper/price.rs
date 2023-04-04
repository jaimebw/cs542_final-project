use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

/// This is done instead of using a floating point value to avoid rounding errors. The value is
/// stored as a number of cents.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PriceUSD(i32);

impl PriceUSD {
    pub fn new(dollars: i32, cents: i32) -> Self {
        PriceUSD(100 * dollars + cents)
    }
}

impl Debug for PriceUSD {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let PriceUSD(cents) = self;
        write!(f, "${}.{:.02}", cents / 100, cents % 100)
    }
}


#[derive(Debug)]
pub struct InvalidPrice;

impl FromStr for PriceUSD {
    type Err = InvalidPrice;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if let Some(value) = s.strip_prefix('$') {
            s = value;
        }

        let (dollars, cents) = match s.split_once('.') {
            Some((dollars, cents)) => {
                let dollars = dollars.parse::<i32>().map_err(|_| InvalidPrice)?;
                let cents = cents.parse::<i32>().map_err(|_| InvalidPrice)?;

                (dollars, cents)
            },
            _ => {
                let dollars = s.parse::<i32>().map_err(|_| InvalidPrice)?;

                (dollars, 0)
            }
        };


        Ok(PriceUSD::new(dollars, cents))
    }
}


