use std::fmt;
use html5ever::rcdom::{self, Handle, NodeData};
use failure::Fallible;

pub trait Find {
    type QueryExecutor: QueryExecutor;

    fn find(&self) -> Self::QueryExecutor;
}

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

    pub fn execute(&mut self) -> Fallible<Option<Handle>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for SingleResultQueryExecutor {
    type Output = Option<Handle>;

    fn execute(&mut self) -> Fallible<Self::Output> {
        let mut results = vec![];
        execute_query(&self.0.handle, &self.0.queries, &mut self.0.depth_limit, &mut results)?;
        Ok(match results {
            ref nodes if nodes.is_empty() => None,
            ref mut nodes => Some(nodes.remove(0))
        })
    }
}

#[derive(Debug, Clone)]
pub struct MultipleResultQueryExecutor(QueryBuilder);

impl MultipleResultQueryExecutor {
    pub fn new(node: Handle) -> MultipleResultQueryExecutor {
        MultipleResultQueryExecutor(QueryBuilder::new(node))
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

    pub fn execute(&mut self) -> Fallible<Vec<Handle>> {
        QueryExecutor::execute(self)
    }
}

impl QueryExecutor for MultipleResultQueryExecutor {
    type Output = Vec<Handle>;

    fn execute(&mut self) -> Fallible<Self::Output> { // TODO: should I impl Find & FindAll for html5ever::Node or should these be wrapped?
        let mut results = vec![];
        execute_query(&self.0.handle, &self.0.queries, &mut self.0.depth_limit, &mut results)?;
        Ok(results)
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

fn execute_query<'node>(node: &Handle, queries: &[QueryType], remaining_depth: &mut Option<usize>, found_nodes: &mut Vec<Handle>) -> Fallible<()> {
    let has_children = {
        !node.children.borrow().is_empty()
    };
    if queries.iter().all(|query| query.matches(&node)) {
        found_nodes.push(node.clone());
    }
    if let Some(ref d) = remaining_depth {
        if *d == 0 || !has_children {
            Ok(())
        } else {
            for child in node.children.borrow().iter() {
                execute_query(child, queries, &mut remaining_depth.map(|d| d - 1), found_nodes)?;
            }
            Ok(())
        }
    } else {
        if has_children {
            for child in node.children.borrow().iter() {
                execute_query(child, queries, remaining_depth, found_nodes)?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}
