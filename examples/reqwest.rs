extern crate reqwest;
extern crate soup;

use soup::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let resp = reqwest::get("https://docs.rs/soup/0.1.0/soup/")?;
    let soup = Soup::from_reader(resp)?;
    let result = soup
        .tag("section")
        .attr("id", "main")
        .find()
        .and_then(|section| {
            section
                .tag("span")
                .attr("class", "in-band")
                .find()
                .map(|span| span.text())
        });
    assert_eq!(result, Some("Crate soup".to_string()));
    Ok(())
}
