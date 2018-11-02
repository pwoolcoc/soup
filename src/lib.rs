//! Inspired by the Python library "BeautifulSoup," `soup` is a layer on top of `html5ever` that
//! aims to provide a slightly different API for querying & manipulating HTML
//!
//! # Examples
//!
//! ```rust
//! # extern crate soup;
//! # use std::error::Error;
//! # use soup::prelude::*;
//!
//! # fn main() -> Result<(), Box<Error>> {
//!
//! let html = r#"
//! <!DOCTYPE html>
//! <html>
//!   <head>
//!     <title>My title</title>
//!   </head>
//!   <body>
//!     <h1>My Heading</h1>
//!     <p>Some text</p>
//!     <p>Some more text</p>
//!   </body>
//! </html>
//! "#;
//!
//! let soup = Soup::new(html);
//!
//! assert_eq!(
//!     soup.find()
//!         .tag("p")
//!         .execute()?
//!         .and_then(|p| p.text()),
//!     Some("Some text".to_string())
//! );
//!
//! #   Ok(())
//! # }
//! ```
extern crate failure;
extern crate html5ever;

use std::io::Read;
use html5ever::{
    parse_document,
    rcdom::{
        RcDom, Handle,
    },
    tendril::TendrilSink,
};
use failure::Fallible;

use crate::{
    find::{
        SingleResultQueryExecutor,
        MultipleResultQueryExecutor,
    },
};

/// This module imports all the important types & traits to use `soup` effectively
pub mod prelude {
    pub use crate::find::{Find, FindAll};
    pub use crate::Soup;
    pub use crate::node_ext::NodeExt;
}

mod find;
mod node_ext;

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
    pub fn from_reader<R: Read>(mut reader: R) -> Fallible<Soup> {
        let dom = parse_document(RcDom::default(), Default::default())
                .from_utf8()
                .read_from(&mut reader)?;
        Ok(Soup {
            handle: dom.document,
        })
    }
}

impl find::Find for Soup {
    type QueryExecutor = SingleResultQueryExecutor;

    fn find(&self) -> Self::QueryExecutor {
        self.handle.find()
    }
}

impl find::FindAll for Soup {
    type QueryExecutor = MultipleResultQueryExecutor;

    fn find_all(&self) -> Self::QueryExecutor {
        self.handle.find_all()
    }
}

impl find::Find for Handle {
    type QueryExecutor = SingleResultQueryExecutor;

    fn find(&self) -> Self::QueryExecutor {
        SingleResultQueryExecutor::new(self.clone())
    }
}

impl find::FindAll for Handle {
    type QueryExecutor = MultipleResultQueryExecutor;

    fn find_all(&self) -> Self::QueryExecutor {
        MultipleResultQueryExecutor::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

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
        let result = soup.find().tag("p").execute().unwrap().unwrap();
        assert_eq!(result.text(), Some("One".to_string()));
    }
}
