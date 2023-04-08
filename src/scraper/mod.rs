use lazy_static::lazy_static;
use regex::Regex;

mod api;
pub mod offer;
pub mod price;
pub mod product;
mod rate_limit;

pub use api::AmazonApi;

pub fn extract_asin(url: &str) -> Option<&str> {
    lazy_static! {
        static ref ASIN_REGEX: Regex = Regex::new(r"/dp/([\dA-Z]{10})/*[^/]*?$").unwrap();
    }

    Some(ASIN_REGEX.captures(url)?.get(1)?.as_str())
}

#[cfg(test)]
#[test]
pub fn test_asin_extraction() {
    assert_eq!(
        extract_asin("https://www.amazon.com/dp/B07VGRJDFY"),
        Some("B07VGRJDFY")
    );

    let long_url = "https://www.amazon.com/Warming-Pets-Removable-Non-Slip-Washable/dp/B096S3QHWL/?_encoding=UTF8&pd_rd_w=DZ6f0&content-id=amzn1.sym.436e9684-a04c-4889-97d3-fc86a74d02fb&pf_rd_p=436e9684-a04c-4889-97d3-fc86a74d02fb&pf_rd_r=C66MWWHFNP8AZJNKEBCS&pd_rd_wg=kFZs4&pd_rd_r=ccf07ba4-b6dd-4daf-8e9e-b499cc9d4f30&ref_=pd_gw_trq_dl&th=1";
    assert_eq!(extract_asin(long_url), Some("B096S3QHWL"));
}
