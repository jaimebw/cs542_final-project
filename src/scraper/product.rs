use lazy_static::lazy_static;
use regex::Regex;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Text};
use std::ops::Deref;
use serde::Serialize;


#[derive(Serialize)]
pub struct Product {
    pub asin: String,
    pub name: String,
    pub department: DepartmentHierarchy,
    pub manufacturer: String,
}

impl<'a> TryFrom<&'a Document> for Product {
    type Error = ItemNotfound;

    fn try_from(document: &'a Document) -> Result<Self, Self::Error> {
        let asin = read_product_info(document, "ASIN")
            .ok_or(ItemNotfound)?
            .to_string();

        let manufacturer = read_product_info(document, "Manufacturer")
            .ok_or(ItemNotfound)?
            .to_string();

        let department = DepartmentHierarchy::try_from(document)?;

        let name = document
            .find(Attr("id", "productTitle"))
            .flat_map(|node| node.find(Text))
            .filter_map(|node| node.as_text())
            .map(|text| text.trim())
            .next()
            .ok_or(ItemNotfound)?
            .to_string();

        Ok(Product {
            asin,
            name,
            manufacturer,
            department,
        })
    }
}

fn read_product_info<'a>(node: &'a Document, key: &str) -> Option<&'a str> {
    node.find(Attr("id", "productDetails_detailBullets_sections1"))
        .flat_map(|node| node.find(Name("tr")))
        .filter(|node| {
            node.find(Name("th"))
                .flat_map(|node| node.find(Text))
                .filter_map(|node| node.as_text())
                .map(|text| text.trim())
                .any(|text| text == key)
        })
        .flat_map(|node| node.find(Name("td")))
        .flat_map(|node| node.find(Text))
        .filter_map(|node| node.as_text())
        .map(|text| text.trim())
        .next()
}

#[derive(Debug)]
pub struct ItemNotfound;

#[derive(Serialize)]
pub struct Department {
    pub name: String,
    pub node: u64,
}

impl Department {
    pub fn url(&self) -> String {
        format!("https://amazon.com/b/?node={}", self.node)
    }
}

impl<'a> TryFrom<Node<'a>> for Department {
    type Error = ItemNotfound;

    fn try_from(value: Node<'a>) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref NODE_REGEX: Regex = Regex::new(r"[?&]node=(\d+)$").unwrap();
        }

        let href = value.attr("href").ok_or(ItemNotfound)?;

        let node = NODE_REGEX
            .captures(href)
            .and_then(|matches| matches.get(1))
            .and_then(|node| node.as_str().parse::<u64>().ok())
            .ok_or(ItemNotfound)?;

        Ok(Department {
            name: value.inner_html().trim().to_string(),
            node,
        })
    }
}

#[derive(Serialize)]
pub struct DepartmentHierarchy {
    departments: Vec<Department>,
}

impl<'a> TryFrom<&'a Document> for DepartmentHierarchy {
    type Error = ItemNotfound;

    fn try_from(value: &'a Document) -> Result<Self, Self::Error> {
        Ok(DepartmentHierarchy {
            departments: value
                .find(Attr("id", "wayfinding-breadcrumbs_feature_div"))
                .flat_map(|node| node.find(Name("a")))
                .filter_map(|node| Department::try_from(node).ok())
                .collect(),
        })
    }
}

impl Deref for DepartmentHierarchy {
    type Target = [Department];

    fn deref(&self) -> &Self::Target {
        &self.departments
    }
}
