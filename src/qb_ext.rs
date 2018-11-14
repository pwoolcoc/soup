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
