use std::{
    fmt,
    marker::PhantomData,
};
use html5ever::rcdom::{self, Handle, NodeData};

use crate::pattern::Pattern;

pub trait Query: Clone {
    fn matches(&self, node: &rcdom::Node) -> bool;
}

#[derive(Clone)]
pub struct TagQuery<P: Clone> {
    inner: P
}

impl<P: Pattern> TagQuery<P> {
    fn new(inner: P) -> TagQuery<P> {
        TagQuery { inner }
    }
}

impl<P: Pattern> Query for TagQuery<P> {
    fn matches(&self, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref name, .. } => {
                self.inner.matches(name.local.as_ref())
            },
            _ => false
        }
    }
}

#[derive(Clone)]
pub struct AttrQuery<K: Clone, V: Clone> {
    key: K,
    value: V,
}

impl<K, V> AttrQuery<K, V>
    where K: Pattern,
          V: Pattern,
{
    fn new(key: K, value: V) -> AttrQuery<K, V> {
        AttrQuery { key, value }
    }
}

impl<K, V> Query for AttrQuery<K, V>
    where K: Pattern,
          V: Pattern,
{
    fn matches(&self, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                let mut iter = attrs.iter();
                if let Some(ref attr) = iter.find(|attr| self.key.matches(attr.name.local.as_ref())) {
                    self.value.matches(attr.value.as_ref())
                } else {
                    false
                }
            },
            _ => false
        }
    }
}

impl Query for () {
    fn matches(&self, _: &rcdom::Node) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct QueryWrapper<'a, T: Query, U: Query> {
    inner: T,
    next: Option<U>,
    _l: PhantomData<&'a ()>,
}

// base case for the QueryWrapper
impl<'a> QueryWrapper<'a, (), ()> {
    fn new() -> QueryWrapper<'a, (), ()> {
        QueryWrapper {
            inner: (),
            next: None as Option<()>,
            _l: PhantomData,
        }
    }
}

impl<'a, T, U, V> QueryWrapper<'a, T, QueryWrapper<'a, U, V>>
    where T: Query + 'a,
          U: Query + 'a,
          V: Query + 'a,
{
    fn wrap(inner: T, query: QueryWrapper<'a, U, V>) -> QueryWrapper<'a, T, QueryWrapper<'a, U, V>> {
        QueryWrapper {
            inner,
            next: Some(query),
            _l: PhantomData,
        }
    }
}

impl<'a, T, U> Query for QueryWrapper<'a, T, U>
    where T: Query + 'a,
          U: Query + 'a,
{
    fn matches(&self, node: &rcdom::Node) -> bool {
        let inner_match = self.inner.matches(node);
        if let Some(ref next) = self.next {
            let next_match = next.matches(node);
            next_match && inner_match
        } else {
            inner_match
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
pub struct QueryBuilder<'a, T: Query + 'a=(), U: Query + 'a=()> {
    handle: Handle,
    queries: QueryWrapper<'a, T, U>,
    limit: Option<usize>,
}

impl<'a, T: Query + 'a, U: Query + 'a> fmt::Debug for QueryBuilder<'a, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "QueryBuilder(«Handle», «Queries»)")
    }
}

impl<'a> QueryBuilder<'a, (), ()> {
    pub(crate) fn new(handle: Handle) -> QueryBuilder<'a, (), ()> {
        QueryBuilder {
            handle,
            queries: QueryWrapper::new(),
            limit: None,
        }
    }
}

impl<'a, T, U> QueryBuilder<'a, T, U>
    where T: Query + 'a,
          U: Query + 'a,
{
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
    pub fn limit(mut self, limit: usize) -> QueryBuilder<'a, T, U> {
        self.limit = Some(limit);
        self
    }

    fn push_query<Q: Query + 'a>(self, query: Q) -> QueryBuilder<'a, Q, QueryWrapper<'a, T, U>> {
        let queries = QueryWrapper::<'a, Q, QueryWrapper<T, U>>::wrap(query, self.queries);
        QueryBuilder {
            handle: self.handle,
            queries: queries,
            limit: self.limit,
        }
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
    pub fn tag<P: Pattern>(self, tag: P) -> QueryBuilder<'a, TagQuery<P>, QueryWrapper<'a, T, U>> {
        self.push_query(TagQuery::new(tag))
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
    pub fn attr<P, Q>(self, name: P, value: Q) -> QueryBuilder<'a, AttrQuery<P, Q>, QueryWrapper<'a, T, U>>
            where P: Pattern,
                  Q: Pattern,
    {
        self.push_query(AttrQuery::new(name, value))
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
    pub fn class<P: Pattern>(self, value: P) -> QueryBuilder<'a, AttrQuery<&'static str, P>, QueryWrapper<'a, T, U>> {
        self.attr("class", value)
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
    pub fn find_all(self) -> BoxNodeIter<'a> {
        self.into_iter()
    }
}


struct NodeIterator<'a, T: Query + 'a, U: Query + 'a> {
    handle: Handle,
    queries: QueryWrapper<'a, T, U>,
    done: bool,
}

impl<'a, T: Query + 'a, U: Query + 'a> NodeIterator<'a, T, U> {
    fn new(handle: Handle, queries: QueryWrapper<'a, T, U>) -> NodeIterator<'a, T, U> {
        NodeIterator {
            handle,
            queries,
            done: false,
        }
    }
}

impl<'a, T, U> Iterator for NodeIterator<'a, T, U>
    where T: Query + 'a,
          U: Query + 'a,
{
    type Item = Option<Handle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        if Query::matches(&self.queries, &self.handle) {
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

type BoxOptionNodeIter<'a> = Box<Iterator<Item=Option<Handle>> + 'a>;
type BoxNodeIter<'a> = Box<Iterator<Item=Handle> + 'a>;


impl<'a, T: Query + 'a, U: Query + 'a> IntoIterator for QueryBuilder<'a, T, U> {
    type Item = Handle;
    type IntoIter = BoxNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = build_iter(self.handle.clone(), self.queries);
        let mut iter = Box::new(iter.flat_map(|node| node)) as BoxNodeIter;
        if let Some(limit) = self.limit {
            iter = Box::new(iter.take(limit)) as BoxNodeIter;
        }
        iter
    }
}

fn build_iter<'a, T: Query + 'a, U: Query + 'a>(handle: Handle, queries: QueryWrapper<'a, T, U>) -> BoxOptionNodeIter<'a> {
    let iter = NodeIterator::new(handle.clone(), queries.clone());
    handle.children.borrow().iter().fold(Box::new(iter) as BoxOptionNodeIter, |acc, child| {
        let child_iter = build_iter(child.clone(), queries.clone());
        Box::new(acc.chain(Box::new(child_iter) as BoxOptionNodeIter)) as BoxOptionNodeIter
    })
}

