use std::{
    collections::BTreeMap,
};
use html5ever::rcdom::{self, Handle, NodeData};

use crate::{
    find::{
        QueryBuilder,
        QueryWrapper,
        TagQuery,
        AttrQuery,
    },
    pattern::Pattern,
};

/// Adds some convenience methods to the `html5ever::rcdom::Node` type
pub trait NodeExt: Sized {
    fn get_node(&self) -> &rcdom::Node;
    fn get_handle(self) -> Handle;

    /// Retrieves the name of the node
    ///
    /// If this node is an element, the name of that element is returned. Otherwise, special names
    /// are used:
    ///
    /// * Document -> "\[document\]"
    /// * Doctype -> "\[doctype\]"
    /// * Text -> "\[text\]"
    /// * Comment -> "\[comment\]"
    /// * ProcessingInstruction -> "\[processing-instruction\]"
    fn name(&self) -> &str {
        let node = self.get_node();
        match node.data {
            NodeData::Document { .. } => "[document]",
            NodeData::Doctype { .. } => "[doctype]",
            NodeData::Text { .. } => "[text]",
            NodeData::Comment { .. } => "[comment]",
            NodeData::ProcessingInstruction { .. } => "[processing-instruction]",
            NodeData::Element { ref name, .. } => {
                name.local.as_ref()
            }
        }
    }

    /// Looks for an attribute named `attr` and returns it's value as a string
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate soup;
    /// # use std::error::Error;
    /// # use soup::prelude::*;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div class="foo bar"></div>"#);
    /// let div = soup.tag("div").find().unwrap();
    /// assert_eq!(div.get("class"), Some("foo bar".to_string()));
    /// #   Ok(())
    /// # }
    /// ```
    fn get(&self, attr: &str) -> Option<String> {
        let node = self.get_node();
        match node.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                for it in attrs.iter() {
                    let name = it.name.local.as_ref();
                    if name.to_lowercase() == attr.to_lowercase() {
                        return Some(it.value.to_string());
                    }
                }
                None
            },
            _ => None,
        }
    }

    /// Returns the node's attributes as a BTreeMap
    fn attrs(&self) -> BTreeMap<String, String> {
        let node = self.get_node();
        match node.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                attrs.iter()
                    .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
                    .collect()
            },
            _ => BTreeMap::new()
        }
    }

    /// Retrieves the text value of this element, as well as it's child elements
    fn text(&self) -> String {
        let node = self.get_node();
        let mut result = vec![];
        extract_text(node, &mut result);
        result.join("")
    }

    // QueryBuilder constructor methods

    /// Starts building a Query, with limit `limit`
    fn limit<'a>(self, limit: usize) -> QueryBuilder<'a, (), ()> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.limit(limit)
    }

    /// Starts building a Query, with tag `tag`
    fn tag<'a, P: Pattern>(self, tag: P) -> QueryBuilder<'a, TagQuery<P>, QueryWrapper<'a, (), ()>> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.tag(tag)
    }

    /// Starts building a Query, with attr `attr`
    fn attr<'a, P, Q>(self, name: P, value: Q) -> QueryBuilder<'a, AttrQuery<P,Q>, QueryWrapper<'a, (), ()>>
            where P: Pattern,
                  Q: Pattern,
    {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.attr(name, value)
    }

    /// Starts building a Query, with class `class`
    fn class<'a, P: Pattern>(self, value: P) -> QueryBuilder<'a, AttrQuery<&'static str, P>, QueryWrapper<'a, (), ()>> {
        let handle = self.get_handle();
        let qb = QueryBuilder::new(handle);
        qb.class(value)
    }

    /// Returns the node as an html tag
    fn display(&self) -> String {
        let node = self.get_node();
        match node.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let c = node.children.borrow().iter().map(|child| child.display()).collect::<Vec<_>>().join("");
                let mut a = attrs.borrow().iter().map(|attr| {
                    format!(r#"{}="{}""#, attr.name.local, attr.value.as_ref())
                }).collect::<Vec<_>>();
                a.sort();
                let a = a.join(" ");
                if a.is_empty() {
                    format!("<{}>{}</{}>", name.local.as_ref(), c, name.local.as_ref())
                } else {
                    format!("<{} {}>{}</{}>", name.local.as_ref(), a, c, name.local.as_ref())
                }
            },
            NodeData::Text { ref contents, .. } => {
                contents.borrow().as_ref().to_string()
            },
            NodeData::Comment { ref contents, .. } => {
                format!("<!--{}-->", contents.as_ref())
            },
            _ => "".to_string()
        }
    }
}

fn extract_text(node: &rcdom::Node, result: &mut Vec<String>) {
    match node.data {
        NodeData::Text { ref contents, .. } => result.push(contents.borrow().to_string()),
        _ => (),
    }
    let children = node.children.borrow();
    for child in children.iter() {
        extract_text(child, result);
    }
}

impl NodeExt for Handle {
    fn get_node(&self) -> &rcdom::Node {
        &*self
    }

    fn get_handle(self) -> Handle {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use crate::prelude::*;

    #[test]
    fn name() {
        let soup = Soup::new("<b>some text</b>");
        let b = soup.tag("b").find().unwrap();
        let name = b.name();
        assert_eq!(name, "b");
    }

    #[test]
    fn get() {
        let soup = Soup::new(r#"<div class="one two"></div>"#);
        let div = soup.tag("div").find().unwrap();
        let class = div.get("class");
        assert_eq!(class, Some("one two".to_string()));
    }

    #[test]
    fn attrs() {
        let soup = Soup::new(r#"<div class="one two" id="some-id"></div>"#);
        let div = soup.tag("div").find().unwrap();
        let attrs = div.attrs();
        let mut expected = BTreeMap::new();
        expected.insert("class".to_string(), "one two".to_string());
        expected.insert("id".to_string(), "some-id".to_string());
        assert_eq!(attrs, expected);
    }

    #[test]
    fn display() {
        let soup = Soup::new(r#"<div class="foo bar" id="baz"></div>"#);
        let div = soup.tag("div").find().unwrap();
        assert_eq!(div.display(), r#"<div class="foo bar" id="baz"></div>"#);

        let soup = Soup::new(r#"<div class="foo bar" id="baz"><b>SOME TEXT</b></div>"#);
        let div = soup.tag("div").find().unwrap();
        assert_eq!(div.display(), r#"<div class="foo bar" id="baz"><b>SOME TEXT</b></div>"#);

        let soup = Soup::new(r#"<div class="foo bar" id="baz"><b>SOME TEXT <!-- and a comment --></b></div>"#);
        let div = soup.tag("div").find().unwrap();
        let b = div.tag("b").find().unwrap();
        assert_eq!(b.display(), r#"<b>SOME TEXT <!-- and a comment --></b>"#);
    }
}
