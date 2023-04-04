use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

/// This is done instead of using a floating point value to avoid rounding errors. The value is
/// stored as a number of cents.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PriceUSD(i64);

impl PriceUSD {
    pub fn new(dollars: i64, cents: i64) -> Self {
        PriceUSD(100 * dollars + cents)
    }
}

impl From<PriceUSD> for f32 {
    fn from(PriceUSD(cents): PriceUSD) -> Self {
        cents as f32 / 100.0
    }
}

impl From<PriceUSD> for f64 {
    fn from(PriceUSD(cents): PriceUSD) -> Self {
        cents as f64 / 100.0
    }
}

#[derive(Debug)]
pub struct InsufficientPrecision;

impl TryFrom<f32> for PriceUSD {
    type Error = InsufficientPrecision;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let cents = value * 100.0;
        if cents as i64 as f32 != cents {
            return Err(InsufficientPrecision);
        }

        Ok(PriceUSD(cents as i64))
    }
}

impl TryFrom<f64> for PriceUSD {
    type Error = InsufficientPrecision;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let cents = value * 100.0;
        if cents as i64 as f64 != cents {
            return Err(InsufficientPrecision);
        }

        Ok(PriceUSD(cents as i64))
    }
}

impl Debug for PriceUSD {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let PriceUSD(cents) = self;
        write!(f, "${}.{:.02}", cents / 100, cents.abs() % 100)
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
                let dollars = dollars.parse::<i64>().map_err(|_| InvalidPrice)?;
                let cents = cents.parse::<i64>().map_err(|_| InvalidPrice)?;

                if cents.abs() >= 100 {
                    return Err(InvalidPrice);
                }

                (dollars, cents)
            }
            _ => {
                let dollars = s.parse::<i64>().map_err(|_| InvalidPrice)?;

                (dollars, 0)
            }
        };

        Ok(PriceUSD::new(dollars, cents))
    }
}
