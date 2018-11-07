//! Inspired by the Python library "BeautifulSoup," `soup` is a layer on top of
//! `html5ever` that aims to provide a slightly different API for querying &
//! manipulating HTML
//!
//! # Examples (inspired by bs4's docs)
//!
//! ```
//! # extern crate soup;
//! # use soup::prelude::*;
//!
//! const THREE_SISTERS: &'static str = r#"
//! <html><head><title>The Dormouse's story</title></head>
//! <body>
//! <p class="title"><b>The Dormouse's story</b></p>
//!
//! <p class="story">Once upon a time there were three little sisters; and their names were
//! <a href="http://example.com/elsie" class="sister" id="link1">Elsie</a>,
//! <a href="http://example.com/lacie" class="sister" id="link2">Lacie</a> and
//! <a href="http://example.com/tillie" class="sister" id="link3">Tillie</a>;
//! and they lived at the bottom of a well.</p>
//!
//! <p class="story">...</p>
//! "#;
//! # fn main() {
//!
//! let soup = Soup::new(THREE_SISTERS);
//!
//! let title = soup.tag("title").find().unwrap();
//! assert_eq!(title.display(), "<title>The Dormouse's story</title>");
//! assert_eq!(title.name(), "title");
//! assert_eq!(title.text(), "The Dormouse's story".to_string());
//! # // assert_eq!(title.parent().name(), "head");
//!
//! let p = soup.tag("p").find().unwrap();
//! assert_eq!(
//!     p.display(),
//!     r#"<p class="title"><b>The Dormouse's story</b></p>"#
//! );
//! assert_eq!(p.get("class"), Some("title".to_string()));
//! let a = soup.tag("a").find().unwrap();
//! assert_eq!(
//!     a.display(),
//!     r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>"#
//! );
//! let a_s = soup.tag("a").find_all().collect::<Vec<_>>();
//! assert_eq!(
//!     a_s.iter()
//!         .map(|a| a.display())
//!         .collect::<Vec<_>>()
//!         .join("\n"),
//!     r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>
//! <a class="sister" href="http://example.com/lacie" id="link2">Lacie</a>
//! <a class="sister" href="http://example.com/tillie" id="link3">Tillie</a>"#
//! );
//!
//! let expected = [
//!     "http://example.com/elsie",
//!     "http://example.com/lacie",
//!     "http://example.com/tillie",
//! ];
//!
//! for (i, link) in soup.tag("a").find_all().enumerate() {
//!     let href = link.get("href").unwrap();
//!     assert_eq!(href, expected[i].to_string());
//! }
//!
//! let text = soup.text();
//! assert_eq!(
//!     text,
//!     r#"The Dormouse's story
//!
//! The Dormouse's story
//!
//! Once upon a time there were three little sisters; and their names were
//! Elsie,
//! Lacie and
//! Tillie;
//! and they lived at the bottom of a well.
//!
//! ...
//! "#
//! );
//! # }
//! ```
//!
//! You can use more than just strings to search for results:
//!
//! ```rust
//! # extern crate regex;
//! # extern crate soup;
//! # use soup::prelude::*;
//! # use std::error::Error;
//! use regex::Regex;
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let soup = Soup::new(r#"<body><p>some text, <b>Some bold text</b></p></body>"#);
//! let results = soup.tag(Regex::new("^b")?)
//!                   .find_all()
//!                   .map(|tag| tag.name().to_string())
//!                   .collect::<Vec<_>>();
//! assert_eq!(results, vec!["body".to_string(), "b".to_string()]);
//! #   Ok(())
//! # }
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    rust_2018_compatibility,
    rust_2018_idioms
)]
extern crate html5ever;
#[cfg(feature = "regex")]
extern crate regex;

use html5ever::{
    parse_document,
    rcdom::{Handle, RcDom},
    tendril::TendrilSink,
};
use std::{
    fmt,
    io::{self, Read},
};

use crate::{
    find::{AttrQuery, QueryWrapper, TagQuery},
    pattern::Pattern,
};

/// This module exports all the important types & traits to use `soup`
/// effectively
pub mod prelude {
    pub use crate::{node_ext::NodeExt, Soup};
}

pub use crate::{find::QueryBuilder, node_ext::NodeExt};

mod find;
mod node_ext;
pub mod pattern;

/// Parses HTML & provides methods to query & manipulate the document
#[derive(Clone)]
pub struct Soup {
    handle: Handle,
}

impl Soup {
    /// Create a new `Soup` instance from a string slice
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use soup::prelude::*;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let html = r#"
    /// <!doctype html>
    /// <html>
    ///   <head>
    ///     <title>page title</title>
    ///   </head>
    ///   <body>
    ///     <h1>Heading</h1>
    ///     <p>Some text</p>
    ///     <p>Some more text</p>
    ///   </body>
    /// </html>
    /// "#;
    ///
    /// let soup = Soup::new(html);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn new(html: &str) -> Soup {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .one(html.as_bytes());
        Soup {
            handle: dom.document,
        }
    }

    /// Create a new `Soup` instance from something that implements `Read`
    ///
    /// This is good for parsing the output of an HTTP response, for example.
    ///
    /// ```rust,no_run
    /// # extern crate reqwest;
    /// # extern crate soup;
    /// # use std::error::Error;
    /// use soup::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<Error>> {
    /// let response = reqwest::get("https://docs.rs/soup")?;
    /// let soup = Soup::from_reader(response)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Soup> {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut reader)?;
        Ok(Soup {
            handle: dom.document,
        })
    }

    /// Starts building a Query, with limit `limit`
    pub fn limit<'a>(&self, limit: usize) -> QueryBuilder<'a, (), ()> {
        let qb = QueryBuilder::new(self.handle.clone());
        qb.limit(limit)
    }

    /// Starts building a Query, with tag `tag`
    pub fn tag<'a, P: Pattern>(
        &self,
        tag: P,
    ) -> QueryBuilder<'a, TagQuery<P>, QueryWrapper<'a, (), ()>> {
        let qb = QueryBuilder::new(self.handle.clone());
        qb.tag(tag)
    }

    /// Starts building a Query, with attr `attr`
    pub fn attr<'a, P, Q>(
        &self,
        name: P,
        value: Q,
    ) -> QueryBuilder<'a, AttrQuery<P, Q>, QueryWrapper<'a, (), ()>>
    where
        P: Pattern,
        Q: Pattern,
    {
        let qb = QueryBuilder::new(self.handle.clone());
        qb.attr(name, value)
    }

    /// Starts building a Query, with class `class`
    pub fn class<'a, P: Pattern>(
        &self,
        value: P,
    ) -> QueryBuilder<'a, AttrQuery<&'static str, P>, QueryWrapper<'a, (), ()>> {
        let qb = QueryBuilder::new(self.handle.clone());
        qb.class(value)
    }

    /// Extracts all text from the HTML
    pub fn text(&self) -> String {
        self.handle.text()
    }
}

impl fmt::Debug for Soup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HTML_STRING: &'static str = r#"
<!doctype html>
<html>
  <head>
    <title>foo</title>
  </head>
  <body>
    <p>One</p>
    <p>Two</p>
  </body>
</html>
"#;

    #[test]
    fn find() {
        let soup = Soup::new(TEST_HTML_STRING);
        let result = soup.tag("p").find().unwrap();
        assert_eq!(result.text(), "One".to_string());
    }

    #[test]
    fn find_all() {
        let soup = Soup::new(TEST_HTML_STRING);
        let result = soup
            .tag("p")
            .find_all()
            .map(|p| p.text())
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["One".to_string(), "Two".to_string()]);
    }
}
