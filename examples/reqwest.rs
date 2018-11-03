extern crate reqwest;
extern crate soup;

use soup::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let resp = reqwest::get("https://docs.rs/soup/0.1.0/soup/")?;
    let soup = Soup::from_reader(resp)?;
    let result = soup.find()
        .tag("section")
        .attr("id", "main")
        .execute()
        .and_then(|section| {
            section.find()
                .tag("span")
                .attr("class", "in-band")
                .execute()
                .and_then(|span| span.text())
        });
    assert_eq!(result, Some("Crate \nsoup".to_string()));
    Ok(())
}
