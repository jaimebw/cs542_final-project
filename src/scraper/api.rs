use crate::scraper::offer::Offer;
use crate::scraper::rate_limit::RateLimit;
use futures::{stream, StreamExt};
use html5ever::tendril::ByteTendril;
use log::{error, warn};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, IntoUrl};
use select::document::Document;
use select::predicate::{Attr, Text};
use std::str::FromStr;
use std::time::Duration;

/// I call it an API, but it is really just a web scraper with helper functions
pub struct AmazonApi {
    client: Client,
    rate_limit: RateLimit,
}

impl Default for AmazonApi {
    fn default() -> Self {
        AmazonApi {
            client: Client::new(),
            rate_limit: RateLimit::new(20, Duration::from_millis(50)),
        }
    }
}

impl AmazonApi {
    async fn get_text<U: IntoUrl>(&self, url: U) -> reqwest::Result<Document> {
        let mut response = self
            .rate_limit
            .perform_rate_limited(move || self.client.get(url).send())
            .await?;

        let is_utf8 = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|x| x.to_str().ok())
            .map(|x| x.ends_with("charset=UTF-8") || x.ends_with("charset=utf-8"))
            .unwrap_or(false);

        if !is_utf8 {
            // If the request is not utf-8 we can let reqwest buffer it for us, but we will have to
            // spend extra time copying the response into the document.
            return Ok(Document::from(response.text().await?.as_str()));
        }

        let mut tendril = ByteTendril::new();
        if let Some(length) = response.content_length() {
            tendril.reserve(length as u32);
        }

        while let Some(chunk) = response.chunk().await? {
            tendril.push_slice(&*chunk);
        }

        match tendril.try_reinterpret() {
            Ok(str_tendril) => Ok(Document::from(str_tendril)),
            Err(tendril) => {
                error!("Request with Content-Type=UTF-8 contained non-utf8 data, performing lossy conversion");
                Ok(Document::from(&*String::from_utf8_lossy(&tendril)))
            }
        }
    }

    pub async fn get_offer_page(&self, asin: &str, page: u32) -> reqwest::Result<Document> {
        assert!(page >= 1);

        // The first page is special because it also includes the header and side-bar
        let url = match page {
            1 => format!("https://www.amazon.com/gp/product/ajax/ref=dp_aod_ALL_mbc?asin={}&m=&qid=&smid=&sourcecustomerorglistid=&sourcecustomerorglistitemid=&sr=&pc=dp&experienceId=aodAjaxMain", asin),
            _ => format!("https://www.amazon.com/gp/product/ajax/ref=aod_page_{0}?asin={1}&pc=dp&isonlyrenderofferlist=true&pageno={0}&experienceId=aodAjaxMain", page, asin),
        };

        self.get_text(url).await
    }

    pub async fn get_offers_for_asin(&self, asin: &str) -> reqwest::Result<Vec<Offer>> {
        const OFFERS_PER_PAGE: u32 = 10;

        let first_page = self.get_offer_page(asin, 1).await?;

        let total_offers = first_page
            .find(Attr("id", "aod-filter-offer-count-string"))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .filter_map(|s| {
                // We are expecting a string of the form "123 options"
                u32::from_str(s.strip_suffix(" options")?).ok()
            })
            .next();

        let total_offers = match total_offers {
            Some(n) => n,
            None => {
                warn!("Failed to find offer count for item {}. Either no offers are present or an error may have occurred.", asin);
                return Ok(Vec::new());
            }
        };

        let mut offer_list = Vec::new();

        for node in first_page.find(Attr("id", "aod-offer")) {
            match Offer::try_from(node) {
                Ok(offer) => offer_list.push(offer),
                Err(err) => warn!("Failed to parse offer for item {}: {:?}", asin, err),
            }
        }

        let num_offer_pages = (total_offers + OFFERS_PER_PAGE - 1) / OFFERS_PER_PAGE;

        let mut offer_pages =
            stream::iter((2..=num_offer_pages).map(|page| self.get_offer_page(asin, page)))
                .buffer_unordered(self.rate_limit.max_sync_usages());

        while let Some(document) = offer_pages.next().await {
            let mut offers_on_page = 0;

            for node in document?.find(Attr("id", "aod-offer")) {
                offers_on_page += 1;
                match Offer::try_from(node) {
                    Ok(offer) => offer_list.push(offer),
                    Err(err) => warn!("Failed to parse offer for item {}: {:?}", asin, err),
                }
            }

            if offers_on_page == 0 {
                warn!("Found no offers on page for item {}. This may indicate that some offers were removed or an error occurred", asin);
            }
        }

        Ok(offer_list)
    }
}
