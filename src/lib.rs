//! HTML Soup
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
//! # assert_eq!(&soup.title().to_string()[..], "My title"); // tag string content
//! # assert_eq!(soup.title().name, "title"); // tag name
//! assert_eq!(
//!     soup.find()
//!         .tag("p")
//!         .execute()?
//!         .and_then(|p| p.text()),
//!     Some("Some text".to_string())
//! );
//!
//! /*
//! assert_eq!(
//!     soup.find_all()
//!         .tag("p")
//!         .execute()?
//!         .iter()
//!         .map(|p| p.to_string())
//!         .collect::<Vec<_>>(),
//!     vec![
//!         "Some text",
//!         "Some more text",
//!     ]
//! );
//! */
//!
//! #   Ok(())
//! # }
//! ```
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

pub mod prelude {
    pub use crate::find::{Find, FindAll};
    pub use crate::Soup;
    pub use crate::node_ext::NodeExt;
}

mod find;
mod node_ext;

#[derive(Clone)]
pub struct Soup {
    handle: Handle,
}

impl Soup {
    pub fn new(html: &str) -> Soup {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .one(html.as_bytes());
        Soup {
            handle: dom.document,
        }
    }

    pub fn from_reader<R: Read>(mut reader: R) -> Fallible<Soup> {
        let dom = parse_document(RcDom::default(), Default::default())
                .from_utf8()
                .read_from(&mut reader)?;
        Ok(Soup {
            handle: dom.document,
        })
    }

    pub fn find(&self) -> SingleResultQueryExecutor {
        // forward this call to the Find impl
        crate::find::Find::find(self)
    }

    pub fn find_all(&self) -> MultipleResultQueryExecutor {
        // forward this call to the FindAll impl
        crate::find::FindAll::find_all(self)
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
