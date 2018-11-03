use std::collections::BTreeMap;
use html5ever::rcdom::{self, Handle, NodeData};
use failure::Fallible;

/// Adds some convenience methods to the `html5ever::rcdom::Node` type
pub trait NodeExt {
    fn get_node(&self) -> &rcdom::Node;

    /// Retrieves the name of the node
    ///
    /// If this node is an element, the name of that element is returned. Otherwise, special names
    /// are used:
    ///
    /// Document -> "[document]"
    /// Doctype -> "[doctype]"
    /// Text -> "[text]"
    /// Comment -> "[comment]"
    /// ProcessingInstruction -> "[processing-instruction]"
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
    /// let div = soup.find().tag("div").execute().unwrap();
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
    fn text(&self) -> Option<String> {
        let node = self.get_node();
        let mut result = vec![];
        match extract_text(node, &mut result) {
            Ok(..) => Some(result.join("\n")),
            Err(..) => None
        }
    }
}

fn extract_text(node: &rcdom::Node, result: &mut Vec<String>) -> Fallible<()> {
    match node.data {
        NodeData::Text { ref contents, .. } => result.push(contents.borrow().to_string()),
        _ => (),
    }
    let children = node.children.borrow();
    for child in children.iter() {
        extract_text(child, result)?;
    }
    Ok(())
}

impl<'node> NodeExt for &'node rcdom::Node {
    fn get_node(&self) -> &rcdom::Node {
        self
    }
}

impl NodeExt for Handle {
    fn get_node(&self) -> &rcdom::Node {
        &*self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use crate::prelude::*;

    #[test]
    fn name() {
        let soup = Soup::new("<b>some text</b>");
        let b = soup.find().tag("b").execute().unwrap();
        let name = b.name();
        assert_eq!(name, "b");
    }

    #[test]
    fn get() {
        let soup = Soup::new(r#"<div class="one two"></div>"#);
        let div = soup.find().tag("div").execute().unwrap();
        let class = div.get("class");
        assert_eq!(class, Some("one two".to_string()));
    }

    #[test]
    fn attrs() {
        let soup = Soup::new(r#"<div class="one two" id="some-id"></div>"#);
        let div = soup.find().tag("div").execute().unwrap();
        let attrs = div.attrs();
        let mut expected = BTreeMap::new();
        expected.insert("class".to_string(), "one two".to_string());
        expected.insert("id".to_string(), "some-id".to_string());
        assert_eq!(attrs, expected);
    }
}
