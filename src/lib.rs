//! Inspired by the Python library "BeautifulSoup," `soup` is a layer on top of
//! `html5ever` that aims to provide a slightly different API for querying &
//! manipulating HTML
//!
//! # Examples (inspired by bs4's docs)
//!
//! Here is the HTML document we will be using for the rest of the examples:
//!
//! ```
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
//! # fn main() {}
//! ```
//!
//! First let's try searching for a tag with a specific name:
//!
//! ```
//! # extern crate soup;
//! # const THREE_SISTERS: &'static str = r#"
//! # <html><head><title>The Dormouse's story</title></head>
//! # <body>
//! # <p class="title"><b>The Dormouse's story</b></p>
//! #
//! # <p class="story">Once upon a time there were three little sisters; and their names were
//! # <a href="http://example.com/elsie" class="sister" id="link1">Elsie</a>,
//! # <a href="http://example.com/lacie" class="sister" id="link2">Lacie</a> and
//! # <a href="http://example.com/tillie" class="sister" id="link3">Tillie</a>;
//! # and they lived at the bottom of a well.</p>
//! #
//! # <p class="story">...</p>
//! # "#;
//! # fn main() {
//! use soup::prelude::*;
//!
//! let soup = Soup::new(THREE_SISTERS);
//!
//! let title = soup.tag("title").find().expect("Couldn't find tag 'title'");
//! assert_eq!(title.display(), "<title>The Dormouse's story</title>");
//! assert_eq!(title.name(), "title");
//! assert_eq!(title.text(), "The Dormouse's story".to_string());
//! assert_eq!(title.parent().expect("Couldn't find parent of 'title'").name(), "head");
//!
//! let p = soup.tag("p").find().expect("Couldn't find tag 'p'");
//! assert_eq!(
//!     p.display(),
//!     r#"<p class="title"><b>The Dormouse's story</b></p>"#
//! );
//! assert_eq!(p.get("class"), Some("title".to_string()));
//! # }
//! ```
//!
//! So we see that `.find` will give us the first element that matches the query, and we've seen some
//! of the methods that we can call on the results. But what if we want to retrieve more than one
//! element with the query? For that, we'll use `.find_all`:
//!
//! ```
//! # extern crate soup;
//! # use soup::prelude::*;
//! # const THREE_SISTERS: &'static str = r#"
//! # <html><head><title>The Dormouse's story</title></head>
//! # <body>
//! # <p class="title"><b>The Dormouse's story</b></p>
//! #
//! # <p class="story">Once upon a time there were three little sisters; and their names were
//! # <a href="http://example.com/elsie" class="sister" id="link1">Elsie</a>,
//! # <a href="http://example.com/lacie" class="sister" id="link2">Lacie</a> and
//! # <a href="http://example.com/tillie" class="sister" id="link3">Tillie</a>;
//! # and they lived at the bottom of a well.</p>
//! #
//! # <p class="story">...</p>
//! # "#;
//! # fn main() {
//! # let soup = Soup::new(THREE_SISTERS);
//! // .find returns only the first 'a' tag
//! let a = soup.tag("a").find().expect("Couldn't find tag 'a'");
//! assert_eq!(
//!     a.display(),
//!     r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>"#
//! );
//! // but .find_all will return _all_ of them:
//! let a_s = soup.tag("a").find_all();
//! assert_eq!(
//!     a_s.map(|a| a.display())
//!        .collect::<Vec<_>>()
//!        .join("\n"),
//!     r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>
//! <a class="sister" href="http://example.com/lacie" id="link2">Lacie</a>
//! <a class="sister" href="http://example.com/tillie" id="link3">Tillie</a>"#
//! );
//! # }
//! ```
//!
//! Since `.find_all` returns an iterator, you can use it with all the methods you would
//! use with other iterators:
//!
//! ```
//! # extern crate soup;
//! # use soup::prelude::*;
//! # const THREE_SISTERS: &'static str = r#"
//! # <html><head><title>The Dormouse's story</title></head>
//! # <body>
//! # <p class="title"><b>The Dormouse's story</b></p>
//! #
//! # <p class="story">Once upon a time there were three little sisters; and their names were
//! # <a href="http://example.com/elsie" class="sister" id="link1">Elsie</a>,
//! # <a href="http://example.com/lacie" class="sister" id="link2">Lacie</a> and
//! # <a href="http://example.com/tillie" class="sister" id="link3">Tillie</a>;
//! # and they lived at the bottom of a well.</p>
//! #
//! # <p class="story">...</p>
//! # "#;
//! # fn main() {
//! # let soup = Soup::new(THREE_SISTERS);
//! let expected = [
//!     "http://example.com/elsie",
//!     "http://example.com/lacie",
//!     "http://example.com/tillie",
//! ];
//!
//! for (i, link) in soup.tag("a").find_all().enumerate() {
//!     let href = link.get("href").expect("Couldn't find link with 'href' attribute");
//!     assert_eq!(href, expected[i].to_string());
//! }
//! # }
//! ```
//!
//! The top-level structure we've been working with here, `soup`, implements the same methods
//! that the query results do, so you can call the same methods on it and it will delegate the
//! calls to the root node:
//!
//! ```
//! # extern crate soup;
//! # use soup::prelude::*;
//! # const THREE_SISTERS: &'static str = r#"
//! # <html><head><title>The Dormouse's story</title></head>
//! # <body>
//! # <p class="title"><b>The Dormouse's story</b></p>
//! #
//! # <p class="story">Once upon a time there were three little sisters; and their names were
//! # <a href="http://example.com/elsie" class="sister" id="link1">Elsie</a>,
//! # <a href="http://example.com/lacie" class="sister" id="link2">Lacie</a> and
//! # <a href="http://example.com/tillie" class="sister" id="link3">Tillie</a>;
//! # and they lived at the bottom of a well.</p>
//! #
//! # <p class="story">...</p>
//! # "#;
//! # fn main() {
//! # let soup = Soup::new(THREE_SISTERS);
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
//! You can use more than just strings to search for results, such as Regex:
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
//! ```
//!
//! Passing `true` will match everything:
//!
//! ```rust
//! # extern crate soup;
//! # use soup::prelude::*;
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let soup = Soup::new(r#"<body><p>some text, <b>Some bold text</b></p></body>"#);
//! let results = soup.tag(true)
//!                   .find_all()
//!                   .map(|tag| tag.name().to_string())
//!                   .collect::<Vec<_>>();
//! assert_eq!(results, vec![
//!     "html".to_string(),
//!     "head".to_string(),
//!     "body".to_string(),
//!     "p".to_string(),
//!     "b".to_string(),
//! ]);
//! #   Ok(())
//! # }
//! ```
//!
//! (also, passing `false` will always return no results, though if that is useful to you, please let me know)
//!
//! So what can you do once you get the result of a query? Well, for one thing, you can traverse the tree a few
//! different ways. You can ascend the tree:
//!
//! ```rust
//! # extern crate soup;
//! # use soup::prelude::*;
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let soup = Soup::new(r#"<body><p>some text, <b>Some bold text</b></p></body>"#);
//! let b = soup.tag("b")
//!             .find()
//!             .expect("Couldn't find tag 'b'");
//! let p = b.parent()
//!          .expect("Couldn't find parent of 'b'");
//! assert_eq!(p.name(), "p".to_string());
//! let body = p.parent()
//!             .expect("Couldn't find parent of 'p'");
//! assert_eq!(body.name(), "body".to_string());
//! #   Ok(())
//! # }
//! ```
//!
//! Or you can descend it:
//!
//! ```rust
//! # extern crate soup;
//! # use soup::prelude::*;
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let soup = Soup::new(r#"<body><ul><li>ONE</li><li>TWO</li><li>THREE</li></ul></body>"#);
//! let ul = soup.tag("ul")
//!             .find()
//!             .expect("Couldn't find tag 'ul'");
//! let mut li_tags = ul.children().filter(|child| child.is_element());
//! assert_eq!(li_tags.next().map(|tag| tag.text().to_string()), Some("ONE".to_string()));
//! assert_eq!(li_tags.next().map(|tag| tag.text().to_string()), Some("TWO".to_string()));
//! assert_eq!(li_tags.next().map(|tag| tag.text().to_string()), Some("THREE".to_string()));
//! assert!(li_tags.next().is_none());
//! #   Ok(())
//! # }
//! ```
//!
//! Or ascend it with an iterator:
//!
//! ```rust
//! # extern crate soup;
//! # use soup::prelude::*;
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let soup = Soup::new(r#"<body><ul><li>ONE</li><li>TWO</li><li>THREE</li></ul></body>"#);
//! let li = soup.tag("li").find().expect("Couldn't find tag 'li'");
//! let mut parents = li.parents();
//! assert_eq!(parents.next().map(|tag| tag.name().to_string()), Some("ul".to_string()));
//! assert_eq!(parents.next().map(|tag| tag.name().to_string()), Some("body".to_string()));
//! assert_eq!(parents.next().map(|tag| tag.name().to_string()), Some("html".to_string()));
//! assert_eq!(parents.next().map(|tag| tag.name().to_string()), Some("[document]".to_string()));
//! #   Ok(())
//! # }
//! ```
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
    rcdom::RcDom,
    tendril::TendrilSink,
};
use std::{
    fmt,
    io::{self, Read},
};

/// This module exports all the important types & traits to use `soup`
/// effectively
pub mod prelude {
    pub use crate::{node_ext::NodeExt, qb_ext::QueryBuilderExt, Soup};
}

pub use crate::{find::QueryBuilder, node_ext::NodeExt, qb_ext::QueryBuilderExt};

mod attribute;
mod find;
mod qb_ext;
mod node_ext;
pub mod pattern;

/// Parses HTML & provides methods to query & manipulate the document
pub struct Soup {
    handle: RcDom,
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
            handle: dom,
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
            handle: dom,
        })
    }

    /// Extracts all text from the HTML
    pub fn text(&self) -> String {
        self.handle.document.text()
    }
}

impl From<RcDom> for Soup {
    fn from(rc: RcDom) -> Soup {
        Soup {
            handle: rc
        }
    }
}

impl fmt::Debug for Soup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.handle.document.text())
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
        let result = soup.tag("p").find().expect("Couldn't find tag 'p'");
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
