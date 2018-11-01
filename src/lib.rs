//! HTML Soup
//!
//! # Examples
//!
//! ```rust
//! # extern crate soup;
//! # use std::error::Error;
//! # use soup::Soup;
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
//! let soup = Soup::new(html);
//!
//! assert_eq!(&soup.title().to_string()[..], "My title"); // tag string content
//! assert_eq!(soup.title().name, "title"); // tag name
//! assert_eq!(&soup.find().tag("p").execute()?.to_string(), "Some text");
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
//!
//! #   Ok(())
//! # }
//! ```
use std::io::Read;
use html5ever::{
    parse_document,
    rcdom::{
        self, RcDom, Handle,
    },
    tendril::TendrilSink,
};
use failure::Fallible;

use crate::find::{
    SingleResultQueryExecutor,
    MultipleResultQueryExecutor,
};

mod find;

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

impl<'node> find::Find<'node> for Soup {
    type QueryExecutor = SingleResultQueryExecutor<'node>;

    fn find(&'node self) -> Self::QueryExecutor {
        self.handle.find()
    }
}

impl<'node> find::FindAll<'node> for Soup {
    type QueryExecutor = MultipleResultQueryExecutor<'node>;

    fn find_all(&'node self) -> Self::QueryExecutor {
        self.handle.find_all()
    }
}

impl<'node> find::Find<'node> for Handle {
    type QueryExecutor = SingleResultQueryExecutor<'node>;

    fn find(&'node self) -> Self::QueryExecutor {
        SingleResultQueryExecutor::new(&*self)
    }
}

impl<'node> find::FindAll<'node> for Handle {
    type QueryExecutor = MultipleResultQueryExecutor<'node>;

    fn find_all(&'node self) -> Self::QueryExecutor {
        MultipleResultQueryExecutor::new(&*self)
    }
}

impl<'node> find::Find<'node> for &'node rcdom::Node {
    type QueryExecutor = SingleResultQueryExecutor<'node>;

    fn find(&'node self) -> Self::QueryExecutor {
        SingleResultQueryExecutor::new(self)
    }
}

impl<'node> find::FindAll<'node> for &'node rcdom::Node {
    type QueryExecutor = MultipleResultQueryExecutor<'node>;

    fn find_all(&'node self) -> Self::QueryExecutor {
        MultipleResultQueryExecutor::new(self)
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
        assert_eq!(&result.to_string()[..], "One");
    }
}
