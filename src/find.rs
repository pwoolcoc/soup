use std::{
    rc::Rc,
    fmt
};
use html5ever::rcdom::{self, Handle, NodeData};

use crate::pattern::Pattern;

enum QueryType {
    Tag(Box<dyn Pattern>),
    Attr(Box<dyn Pattern>, Box<dyn Pattern>),
}

impl QueryType {
    fn matches(&self, node: &rcdom::Node) -> bool {
        match self {
            QueryType::Tag(ref s) => self.match_tag(s, node),
            QueryType::Attr(ref k, ref v) => self.match_attr(k, v, node),
        }
    }

    fn match_tag(&self, tag: &Box<dyn Pattern>, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref name, .. } => {
                tag.matches(name.local.as_ref())
            },
            _ => false
        }
    }

    fn match_attr(&self, key: &Box<dyn Pattern>, value: &Box<dyn Pattern>, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                let mut iter = attrs.iter();
                if let Some(ref attr) = iter.find(|attr| key.matches(attr.name.local.as_ref())) {
                    value.matches(attr.value.as_ref())
                } else {
                    false
                }
            },
            _ => false
        }
    }
}

/// Construct a query to apply to an HTML tree
///
/// # Example
///
/// ```rust
/// # extern crate soup;
/// # use soup::prelude::*;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<Error>> {
/// let soup = Soup::new(r#"<div id="foo">BAR</div><div id="baz">QUUX</div>"#);
/// let query = soup.tag("div")         // result must be a div
///                 .attr("id", "foo")  // with id "foo"
///                 .find();            // executes the query, returns the first result
/// #   Ok(())
/// # }
#[derive(Clone)]
pub struct QueryBuilder {
    handle: Handle,
    queries: Vec<Rc<QueryType>>,
    limit: Option<usize>,
}

impl fmt::Debug for QueryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "QueryBuilder(«Handle», «Queries»)")
    }
}

impl QueryBuilder {
    pub(crate) fn new(handle: Handle) -> QueryBuilder {
        QueryBuilder {
            handle,
            queries: vec![],
            limit: None,
        }
    }

    /// Adds a limit to the number of results that can be returned
    ///
    /// This method adds an upper bound to the number of results that will be returned by the query
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div id="one"></div><div id="two"></div><div id="three></div>"#);
    /// let results = soup.tag("div").limit(2).find_all().collect::<Vec<_>>();
    /// assert_eq!(results.len(), 2);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn limit(&mut self, limit: usize) -> &mut QueryBuilder {
        self.limit = Some(limit);
        self
    }

    /// Specifies a tag for which to search
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#);
    /// let result = soup.tag("b").find().unwrap();
    /// assert_eq!(result.get("id"), Some("bold-tag".to_string()));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn tag<P: 'static + Pattern>(&mut self, tag: P) -> &mut QueryBuilder {
        self.queries.push(Rc::new(QueryType::Tag(Box::new(tag))));
        self
    }

    /// Specifies an attribute name/value pair for which to search
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div>Test</div><section><b id="bold-tag">SOME BOLD TEXT</b></section>"#);
    /// let result = soup.attr("id", "bold-tag").find().unwrap();
    /// assert_eq!(result.name(), "b".to_string());
    /// #   Ok(())
    /// # }
    /// ```
    pub fn attr<P, Q>(&mut self, name: P, value: Q) -> &mut QueryBuilder
            where P: 'static + Pattern,
                  Q: 'static + Pattern,
    {
        self.queries.push(Rc::new(QueryType::Attr(Box::new(name), Box::new(value))));
        self
    }

    /// Specifies a class name for which to search 
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div>Test</div><section class="content"><b id="bold-tag">SOME BOLD TEXT</b></section>"#);
    /// let result = soup.class("content").find().unwrap();
    /// assert_eq!(result.name(), "section".to_string());
    /// #   Ok(())
    /// # }
    /// ```
    pub fn class<P: 'static + Pattern>(&mut self, value: P) -> &mut QueryBuilder {
        self.attr("class", value);
        self
    }

    /// Executes the query, and returns either the first result, or `None`
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#);
    /// let result = soup.tag("li").find().unwrap();
    /// assert_eq!(result.get("id"), Some("one".to_string()));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn find(&mut self) -> Option<Handle> {
        self.limit = Some(1);
        self.clone().into_iter().nth(0)
    }

    /// Executes the query, and returns an iterator of the results
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<ul><li id="one">One</li><li id="two">Two</li><li id="three">Three</li></ul>"#);
    /// let results = soup.tag("li").find_all().collect::<Vec<_>>();
    /// assert_eq!(results.len(), 3);
    /// assert_eq!(results[0].display(), "<li id=\"one\">One</li>");
    /// assert_eq!(results[1].display(), "<li id=\"two\">Two</li>");
    /// assert_eq!(results[2].display(), "<li id=\"three\">Three</li>");
    /// #   Ok(())
    /// # }
    /// ```
    pub fn find_all(&self) -> BoxNodeIter {
        self.clone().into_iter()
    }
}

struct NodeIterator {
    handle: Handle,
    queries: Vec<Rc<QueryType>>,
    done: bool,
}

impl NodeIterator {
    fn new(handle: Handle, queries: Vec<Rc<QueryType>>) -> NodeIterator {
        NodeIterator {
            handle,
            queries,
            done: false,
        }
    }
}

impl Iterator for NodeIterator {
    type Item = Option<Handle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        if self.queries.iter().all(|query| query.matches(&self.handle)) {
            self.done = true;
            Some(Some(self.handle.clone()))
        } else {
            self.done = true;
            Some(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(1))
    }
}

type BoxOptionNodeIter = Box<Iterator<Item=Option<Handle>>>;
type BoxNodeIter = Box<Iterator<Item=Handle>>;

impl IntoIterator for QueryBuilder {
    type Item = Handle;
    type IntoIter = BoxNodeIter;

    fn into_iter(self) -> Self::IntoIter {
        let iter = build_iter(self.handle.clone(), self.queries);
        let mut iter = Box::new(iter.flat_map(|node| node)) as BoxNodeIter;
        if let Some(limit) = self.limit {
            iter = Box::new(iter.take(limit)) as BoxNodeIter;
        }
        iter
    }
}

fn build_iter(handle: Handle, queries: Vec<Rc<QueryType>>) -> BoxOptionNodeIter {
    let iter = NodeIterator::new(handle.clone(), queries.clone());
    handle.children.borrow().iter().fold(Box::new(iter) as BoxOptionNodeIter, |acc, child| {
        let child_iter = build_iter(child.clone(), queries.clone());
        Box::new(acc.chain(Box::new(child_iter) as BoxOptionNodeIter)) as BoxOptionNodeIter
    })
}

