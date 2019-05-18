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
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// use soup::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// let soup = Soup::new(r#"<ul><li>ONE</li><li>TWO</li><li>THREE</li></ul>"#);
    /// let ul = soup.tag("ul").find().expect("couldn't find 'ul'");
    /// let li_text = ul.children()
    ///                 .filter(|node| node.is_element())
    ///                 .map(|node| node.text().to_string())
    ///                 .collect::<Vec<_>>();
    /// assert_eq!(li_text, vec!["ONE".to_string(), "TWO".to_string(), "THREE".to_string()]);
    /// #   Ok(())
    /// # }
    /// ```
    fn children(&self) -> NodeChildIter {
        let handle = self.get_handle();
        NodeChildIter::new(handle.clone())
    }

    /// Iterator over the parents of a node
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// use soup::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// let soup = Soup::new(r#"<div><p><b>FOO</b></p></div>"#);
    /// let b = soup.tag("b").find().expect("Couldn't find tag 'b'");
    /// let parents = b.parents().map(|node| node.name().to_string()).collect::<Vec<_>>();
    /// assert_eq!(parents, vec!["p".to_string(), "div".to_string(), "body".to_string(), "html".to_string(), "[document]".to_string()]);
    /// #   Ok(())
    /// # }
    /// ```
    fn parents(&self) -> NodeParentIter {
        NodeParentIter::new(self.get_handle().clone())
    }
}

/// Iterator over the parents of a node
pub struct NodeParentIter {
    inner: Handle,
}

impl fmt::Debug for NodeParentIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::node_ext::NodeExt;
        f.debug_struct("NodeParentIter")
            .field("inner", &format!("{}", self.inner.display()))
            .finish()
    }
}

impl NodeParentIter {
    pub fn new(handle: Handle) -> NodeParentIter {
        NodeParentIter { inner: handle }
    }
}

impl Iterator for NodeParentIter {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::node_ext::NodeExt;
        if let Some(ref parent) = self.inner.parent() {
            self.inner = parent.clone();
            Some(parent.clone())
        } else {
            None
        }
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
        self.handle.document.clone()
    }
}
