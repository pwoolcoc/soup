use std::fmt;
use html5ever::rcdom::Handle;

use crate::{
    Soup,
    find::{AttrQuery, QueryBuilder, QueryWrapper, TagQuery},
    pattern::Pattern,
};

/// Adds the QueryBuilder constructor methods to the implementing type
pub trait QueryBuilderExt {
    /// Retrieves the Handle that these methods will work on
    fn get_handle(&self) -> Handle;

    // QueryBuilder constructor methods
    /// Starts building a Query, with limit `limit`
    fn limit<'a>(&self, limit: usize) -> QueryBuilder<'a, (), ()> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.limit(limit)
    }

    /// Starts building a Query, with tag `tag`
    fn tag<'a, P: Pattern>(
        &self,
        tag: P,
    ) -> QueryBuilder<'a, TagQuery<P>, QueryWrapper<'a, (), ()>> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.tag(tag)
    }

    /// Starts building a Query, with attr name `name`
    fn attr_name<'a, P>(&self, name: P) -> QueryBuilder<'a, AttrQuery<P, bool>, QueryWrapper<'a, (), ()>>
    where
        P: Pattern
    {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.attr_name(name)
    }

    /// Starts building a Query, with attr value `value`
    fn attr_value<'a, P>(&self, value: P) -> QueryBuilder<'a, AttrQuery<bool, P>, QueryWrapper<'a, (), ()>>
    where
        P: Pattern
    {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.attr_value(value)
    }

    /// Starts building a Query, with attr `attr`
    fn attr<'a, P, Q>(
        &self,
        name: P,
        value: Q,
    ) -> QueryBuilder<'a, AttrQuery<P, Q>, QueryWrapper<'a, (), ()>>
    where
        P: Pattern,
        Q: Pattern,
    {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.attr(name, value)
    }

    /// Starts building a Query, with class `class`
    fn class<'a, P: Pattern>(
        &self,
        value: P,
    ) -> QueryBuilder<'a, AttrQuery<&'static str, P>, QueryWrapper<'a, (), ()>> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.class(value)
    }

    /// Starts building a Query, with recursion set to `recursive`
    fn recursive<'a>(&self, recursive: bool) -> QueryBuilder<'a, (), ()> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.recursive(recursive)
    }

    /// Returns an iterator over the node's children
    fn children(&self) -> NodeChildIter {
        let handle = self.get_handle();
        NodeChildIter::new(handle.clone())
    }
}

/// Iterator over the children of a node
pub struct NodeChildIter {
    inner: Handle,
    idx: usize,
}

impl NodeChildIter {
    pub fn new(handle: Handle) -> NodeChildIter {
        NodeChildIter {
            inner: handle,
            idx: 0,
        }
    }

    fn len(&self) -> usize {
        self.inner.children.borrow().len()
    }
}

impl Iterator for NodeChildIter {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.children.borrow().get(self.idx).cloned();
        self.idx += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl fmt::Debug for NodeChildIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeChildIter")
            .field("handle", &"<handle>")
            .field("idx", &self.idx)
            .finish()
    }
}

impl QueryBuilderExt for Handle {
    fn get_handle(&self) -> Handle {
        self.clone()
    }
}

impl QueryBuilderExt for Soup {
    fn get_handle(&self) -> Handle {
        self.handle.clone()
    }
}
