use std::fmt;
use html5ever::rcdom::{self, Handle, NodeData};
use failure::Fallible;

/// Builds a query that returns the first element that matches
///
/// # Example
///
/// ```
/// # extern crate soup;
/// # use std::error::Error;
/// # use soup::prelude::*;
/// # fn main() -> Result<(), Box<Error>> {
/// let html = r#"
/// <!doctype html>
/// <html>
///   <body>
///     <p>First paragraph</p>
///     <p>Second paragraph</p>
///   </body>
/// </html>
/// "#;
/// let soup = Soup::new(html);
/// assert_eq!(
///     soup.find().tag("p").execute()?.and_then(|p| p.text()),
///     Some("First paragraph".to_string())
/// );
/// #   Ok(())
/// # }
/// ```
pub trait Find {
    type QueryExecutor: QueryExecutor;

    fn find(&self) -> Self::QueryExecutor;
}

/// Builds a query that returns all matching elements
///
/// # Example
///
/// ```
/// # extern crate soup;
/// # use std::error::Error;
/// # use soup::prelude::*;
/// # fn main() -> Result<(), Box<Error>> {
/// let html = r#"
/// <!doctype html>
/// <html>
///   <body>
///     <p>First paragraph</p>
///     <p>Second paragraph</p>
///   </body>
/// </html>
/// "#;
/// let soup = Soup::new(html);
/// assert_eq!(
///     soup.find_all()
///         .tag("p")
///         .execute()?
///         .iter()
///         .map(|p| p.text())
///         .collect::<Vec<_>>(),
///     vec![Some("First paragraph".to_string()), Some("Second paragraph".to_string())]
/// );
/// #   Ok(())
/// # }
/// ```
pub trait FindAll {
    type QueryExecutor: QueryExecutor;

    fn find_all(&self) -> Self::QueryExecutor;
}

#[derive(Debug, Clone, PartialEq)]
enum QueryType {
    Tag(String),
    Attr(String, String),
}

impl QueryType {
    fn matches(&self, node: &rcdom::Node) -> bool {
        match self {
            QueryType::Tag(ref s) => self.match_tag(s, node),
            QueryType::Attr(ref k, ref v) => self.match_attr(k, v, node),
        }
    }

    fn match_tag(&self, tag: &str, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref name, .. } => {
                tag == name.local.as_ref()
            },
            _ => false
        }
    }

    fn match_attr(&self, key: &str, value: &str, node: &rcdom::Node) -> bool {
        match node.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                let mut iter = attrs.iter();
                if let Some(ref attr) = iter.find(|attr| attr.name.local.as_ref() == key) {
                    attr.value.as_ref() == value
                } else {
                    false
                }
            },
            _ => false
        }
    }
}

pub trait QueryExecutor {
    type Output;

    fn execute(&mut self) -> Fallible<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct SingleResultQueryExecutor(QueryBuilder);

impl SingleResultQueryExecutor {
    pub fn new(node: Handle) -> SingleResultQueryExecutor {
        SingleResultQueryExecutor(QueryBuilder::new(node))
    }

    // forward these calls to the underlying builder
    pub fn tag(&mut self, tag: &str) -> &mut Self {
        self.0.tag(tag);
        self
    }
    pub fn attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.0.attr(name, value);
        self
    }
    pub fn class(&mut self, value: &str) -> &mut Self {
        self.0.class(value);
        self
    }

    pub fn execute(&mut self) -> Fallible<Option<Handle>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for SingleResultQueryExecutor {
    type Output = Option<Handle>;

    fn execute(&mut self) -> Fallible<Self::Output> {
        let result = execute_query(&self.0.handle, &self.0.queries, Some(1)).nth(0);
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct MultipleResultQueryExecutor(QueryBuilder);

impl MultipleResultQueryExecutor {
    pub fn new(node: Handle) -> MultipleResultQueryExecutor {
        MultipleResultQueryExecutor(QueryBuilder::new(node))
    }

    // forward these calls to the underlying builder
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.0.limit(limit);
        self
    }

    pub fn tag(&mut self, tag: &str) -> &mut Self {
        self.0.tag(tag);
        self
    }
    pub fn attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.0.attr(name, value);
        self
    }
    pub fn class(&mut self, value: &str) -> &mut Self {
        self.0.class(value);
        self
    }

    pub fn execute(&mut self) -> Fallible<Vec<Handle>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for MultipleResultQueryExecutor {
    type Output = Vec<Handle>;

    fn execute(&mut self) -> Fallible<Self::Output> { // TODO: should I impl Find & FindAll for html5ever::Node or should these be wrapped?
        let results = execute_query(&self.0.handle, &self.0.queries, self.0.limit).collect::<Vec<_>>();
        Ok(results)
    }
}

#[derive(Clone)]
pub struct QueryBuilder {
    handle: Handle,
    queries: Vec<QueryType>,
    limit: Option<usize>,
}

impl fmt::Debug for QueryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "QueryBuilder(Handle, {:?})", self.queries)
    }
}

impl QueryBuilder {
    fn new(handle: Handle) -> QueryBuilder {
        QueryBuilder {
            handle,
            queries: vec![],
            limit: None,
        }
    }

    pub fn limit(&mut self, limit: usize) -> &mut QueryBuilder {
        self.limit = Some(limit);
        self
    }

    pub fn tag(&mut self, tag: &str) -> &mut QueryBuilder {
        self.queries.push(QueryType::Tag(tag.to_string()));
        self
    }

    pub fn attr(&mut self, name: &str, value: &str) -> &mut QueryBuilder {
        self.queries.push(QueryType::Attr(name.to_string(), value.to_string()));
        self
    }

    pub fn class(&mut self, value: &str) -> &mut QueryBuilder {
        self.attr("class", value);
        self
    }
}

struct NodeIterator<'query> {
    handle: Handle,
    queries: &'query [QueryType],
    done: bool,
}

impl<'query> NodeIterator<'query> {
    fn new(handle: Handle, queries: &'query [QueryType]) -> NodeIterator<'query> {
        NodeIterator {
            handle,
            queries,
            done: false,
        }
    }
}

impl<'query> Iterator for NodeIterator<'query> {
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

type BoxOptionNodeIter<'a> = Box<Iterator<Item=Option<Handle>> + 'a>;
type BoxNodeIter<'a> = Box<Iterator<Item=Handle> + 'a>;

fn build_iter<'query>(handle: Handle, queries: &'query [QueryType]) -> BoxOptionNodeIter<'query> {
    let iter = NodeIterator::new(handle.clone(), queries);
    handle.children.borrow().iter().fold(Box::new(iter) as BoxOptionNodeIter, |acc, child| {
        let child_iter = build_iter(child.clone(), queries);
        Box::new(acc.chain(Box::new(child_iter) as BoxOptionNodeIter)) as BoxOptionNodeIter
    })
}

fn execute_query<'query>(node: &Handle, queries: &'query [QueryType], limit: Option<usize>) -> BoxNodeIter<'query> {
    let iter = build_iter(node.clone(), queries);
    let mut iter = Box::new(iter.flat_map(|node| node)) as BoxNodeIter;
    if let Some(limit) = limit {
        iter = Box::new(iter.take(limit)) as BoxNodeIter;
    }
    iter
}
