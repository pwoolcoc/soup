use html5ever::rcdom::{self, Handle, NodeData};
use std::collections::BTreeMap;

/// Adds some convenience methods to the `html5ever::rcdom::Node` type
pub trait NodeExt: Sized {
    /// Retrieves the node that these methods will work on
    fn get_node(&self) -> &rcdom::Node;

    /// Returns `true` if node is of type Document
    fn is_document(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::Document { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if node is of type Doctype
    fn is_doctype(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::Doctype { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if node is of type Text
    fn is_text(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::Text { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if node is of type Comment
    fn is_comment(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::Comment { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if node is of type ProcessingInstruction
    fn is_processing_instruction(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::ProcessingInstruction { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if node is of type Element
    fn is_element(&self) -> bool {
        let node = self.get_node();
        match node.data {
            NodeData::Element { .. } => true,
            _ => false,
        }
    }

    /// Retrieves the name of the node
    ///
    /// If this node is an element, the name of that element is returned.
    /// Otherwise, special names are used:
    ///
    /// * Document -> "\[document\]"
    /// * Doctype -> "\[doctype\]"
    /// * Text -> "\[text\]"
    /// * Comment -> "\[comment\]"
    /// * ProcessingInstruction -> "\[processing-instruction\]"
    fn name(&self) -> &str {
        let node = self.get_node();
        match node.data {
            NodeData::Document {
                ..
            } => "[document]",
            NodeData::Doctype {
                ..
            } => "[doctype]",
            NodeData::Text {
                ..
            } => "[text]",
            NodeData::Comment {
                ..
            } => "[comment]",
            NodeData::ProcessingInstruction {
                ..
            } => "[processing-instruction]",
            NodeData::Element {
                ref name, ..
            } => name.local.as_ref(),
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
    /// let div = soup.tag("div").find().expect("Couldn't find div");
    /// assert_eq!(div.get("class"), Some("foo bar".to_string()));
    /// #   Ok(())
    /// # }
    /// ```
    fn get(&self, attr: &str) -> Option<String> {
        let node = self.get_node();
        match node.data {
            NodeData::Element {
                ref attrs, ..
            } => {
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
            NodeData::Element {
                ref attrs, ..
            } => {
                let attrs = attrs.borrow();
                attrs
                    .iter()
                    .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
                    .collect()
            },
            _ => BTreeMap::new(),
        }
    }

    /// Retrieves the text value of this element, as well as it's child elements
    fn text(&self) -> String {
        let node = self.get_node();
        let mut result = vec![];
        extract_text(node, &mut result);
        result.join("")
    }

    /// Returns the node as an html tag
    fn display(&self) -> String {
        let node = self.get_node();
        match node.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let c = node
                    .children
                    .borrow()
                    .iter()
                    .map(|child| child.display())
                    .collect::<Vec<_>>()
                    .join("");
                let mut a = attrs
                    .borrow()
                    .iter()
                    .map(|attr| format!(r#"{}="{}""#, attr.name.local, attr.value.as_ref()))
                    .collect::<Vec<_>>();
                a.sort();
                let a = a.join(" ");
                if a.is_empty() {
                    format!("<{}>{}</{}>", name.local.as_ref(), c, name.local.as_ref())
                } else {
                    format!(
                        "<{} {}>{}</{}>",
                        name.local.as_ref(),
                        a,
                        c,
                        name.local.as_ref()
                    )
                }
            },
            NodeData::Text {
                ref contents, ..
            } => contents.borrow().as_ref().to_string(),
            NodeData::Comment {
                ref contents, ..
            } => format!("<!--{}-->", contents.as_ref()),
            _ => "".to_string(),
        }
    }

    /// Navigates to the parent of the node, if there is one
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate soup;
    ///
    /// use soup::prelude::*;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<Error>> {
    /// let soup = Soup::new(r#"<div id=""><b>FOO</b></div>"#);
    /// let b = soup.tag("b").find().expect("Couldn't find tag 'b'");
    /// let div = b.parent().expect("Couldn't get parent of tag 'b'");
    /// assert_eq!(div.name(), "div".to_string());
    /// #   Ok(())
    /// # }
    /// ```
    fn parent(&self) -> Option<Handle> {
        let node = self.get_node();
        let parent = node.parent.take(); // leaves node.parent as Cell(None)
        let parent_node = parent.clone();
        node.parent.set(parent); // puts original parent back?
        parent_node.and_then(|node| node.upgrade())
    }
}

fn extract_text(node: &rcdom::Node, result: &mut Vec<String>) {
    match node.data {
        NodeData::Text {
            ref contents, ..
        } => result.push(contents.borrow().to_string()),
        _ => (),
    }
    let children = node.children.borrow();
    for child in children.iter() {
        extract_text(child, result);
    }
}

impl NodeExt for Handle {
    #[inline(always)]
    fn get_node(&self) -> &rcdom::Node {
        &*self
    }
}

impl<'node> NodeExt for &'node rcdom::Node {
    #[inline(always)]
    fn get_node(&self) -> &rcdom::Node {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::collections::BTreeMap;

    #[test]
    fn name() {
        let soup = Soup::new("<b>some text</b>");
        let b = soup.tag("b").find().expect("Couldn't find tag 'b'");
        let name = b.name();
        assert_eq!(name, "b");
    }

    #[test]
    fn get() {
        let soup = Soup::new(r#"<div class="one two"></div>"#);
        let div = soup.tag("div").find().expect("Couldn't find tag 'div'");
        let class = div.get("class");
        assert_eq!(class, Some("one two".to_string()));
    }

    #[test]
    fn attrs() {
        let soup = Soup::new(r#"<div class="one two" id="some-id"></div>"#);
        let div = soup.tag("div").find().expect("Couldn't find tag 'div'");
        let attrs = div.attrs();
        let mut expected = BTreeMap::new();
        expected.insert("class".to_string(), "one two".to_string());
        expected.insert("id".to_string(), "some-id".to_string());
        assert_eq!(attrs, expected);
    }

    #[test]
    fn case_sensitive() {
        let soup = Soup::new(r#"<div class="ONE TWO"></div>"#);
        let one = soup.attr("class", "ONE").find();
        assert!(one.is_some());
        let one = soup.attr("class", "one").find();
        assert!(one.is_none());
    }

    #[test]
    fn display() {
        let soup = Soup::new(r#"<div class="foo bar" id="baz"></div>"#);
        let div = soup.tag("div").find().expect("Couldn't find tag 'div'");
        assert_eq!(div.display(), r#"<div class="foo bar" id="baz"></div>"#);

        let soup = Soup::new(r#"<div class="foo bar" id="baz"><b>SOME TEXT</b></div>"#);
        let div = soup.tag("div").find().expect("Couldn't find tag 'div'");
        assert_eq!(
            div.display(),
            r#"<div class="foo bar" id="baz"><b>SOME TEXT</b></div>"#
        );

        let soup = Soup::new(
            r#"<div class="foo bar" id="baz"><b>SOME TEXT <!-- and a comment --></b></div>"#,
        );
        let div = soup.tag("div").find().expect("Couldn't find tag 'div'");
        let b = div.tag("b").find().expect("Couldn't find tag 'b'");
        assert_eq!(b.display(), r#"<b>SOME TEXT <!-- and a comment --></b>"#);
    }
}
