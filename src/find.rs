use std::fmt;
use html5ever::rcdom::{self, Handle};
use failure::Fallible;

pub(crate) trait Find {
    type QueryExecutor: QueryExecutor;

    fn find(&self) -> Self::QueryExecutor;
}

pub(crate) trait FindAll {
    type QueryExecutor: QueryExecutor;

    fn find_all(&self) -> Self::QueryExecutor;
}

#[derive(Debug, Clone, PartialEq)]
enum QueryType {
    Tag(String),
    Attr(String, String),
}

pub trait QueryExecutor {
    type Output;

    fn execute(&mut self) -> Fallible<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct SingleResultQueryExecutor(QueryBuilder);

impl SingleResultQueryExecutor {
    pub fn new(handle: Handle) -> SingleResultQueryExecutor {
        SingleResultQueryExecutor(QueryBuilder::new(handle))
    }

    // forward these calls to the underlying builder
    pub fn max_depth(&mut self, depth: usize) -> &mut Self {
        self.0.max_depth(depth);
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

    pub fn execute(&mut self) -> Fallible<Option<rcdom::Node>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for SingleResultQueryExecutor {
    type Output = Option<rcdom::Node>;

    fn execute(&mut self) -> Fallible<Self::Output> {
        Ok(match execute_query(&*self.0.handle, &self.0.queries, &mut self.0.depth_limit)? {
            ref nodes if nodes.is_empty() => None,
            ref mut nodes => Some(nodes.remove(0))
        })
    }
}

#[derive(Debug, Clone)]
pub struct MultipleResultQueryExecutor(QueryBuilder);

impl MultipleResultQueryExecutor {
    pub fn new(handle: Handle) -> MultipleResultQueryExecutor {
        MultipleResultQueryExecutor(QueryBuilder::new(handle))
    }

    // forward these calls to the underlying builder
    pub fn max_depth(&mut self, depth: usize) -> &mut Self {
        self.0.max_depth(depth);
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

    pub fn execute(&mut self) -> Fallible<Vec<rcdom::Node>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for MultipleResultQueryExecutor {
    type Output = Vec<rcdom::Node>;

    fn execute(&mut self) -> Fallible<Self::Output> { // TODO: should I impl Find & FindAll for html5ever::Node or should these be wrapped?
        Ok(execute_query(&*self.0.handle, &self.0.queries, &mut self.0.depth_limit)?)
    }
}

#[derive(Clone)]
pub struct QueryBuilder {
    handle: Handle,
    queries: Vec<QueryType>,
    depth_limit: Option<usize>,
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
            depth_limit: None,
        }
    }

    pub fn max_depth(&mut self, depth: usize) -> &mut QueryBuilder {
        self.depth_limit = Some(depth);
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

fn execute_query(node: &rcdom::Node, queries: &[QueryType], remaining_depth: &mut Option<usize>) -> Fallible<Vec<rcdom::Node>> {
    let mut has_children = false;
    let mut found_nodes = vec![];
    if let Some(ref d) = remaining_depth {
        if *d == 0 {
            Ok(found_nodes)
        } else {
            let next_level = execute_query(node, queries, &mut remaining_depth.map(|d| d - 1))?;
            found_nodes.extend(next_level);
            Ok(found_nodes)
        }
    } else {
        if has_children {
            let next_level = execute_query(node, queries, remaining_depth)?;
            found_nodes.extend(next_level);
            Ok(found_nodes)
        } else {
            Ok(found_nodes)
        }
    }
}
