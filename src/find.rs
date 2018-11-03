use std::fmt;
use html5ever::rcdom::{self, Handle, NodeData};

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
    pub(crate) fn new(handle: Handle) -> QueryBuilder {
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

    pub fn find(&mut self) -> Option<Handle> {
        self.limit = Some(1);
        self.clone().into_iter().nth(0)
    }

    pub fn find_all(&self) -> BoxNodeIter {
        self.clone().into_iter()
    }
}

struct NodeIterator {
    handle: Handle,
    queries: Vec<QueryType>,
    done: bool,
}

impl NodeIterator {
    fn new(handle: Handle, queries: Vec<QueryType>) -> NodeIterator {
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

fn build_iter(handle: Handle, queries: Vec<QueryType>) -> BoxOptionNodeIter {
    let iter = NodeIterator::new(handle.clone(), queries.clone());
    handle.children.borrow().iter().fold(Box::new(iter) as BoxOptionNodeIter, |acc, child| {
        let child_iter = build_iter(child.clone(), queries.clone());
        Box::new(acc.chain(Box::new(child_iter) as BoxOptionNodeIter)) as BoxOptionNodeIter
    })
}

