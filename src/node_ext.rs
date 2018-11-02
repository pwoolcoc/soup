use html5ever::rcdom::{self, Handle, NodeData};
use failure::Fallible;

/// Adds some convenience methods to the `html5ever::rcdom::Node` type
pub trait NodeExt {
    fn get_node(&self) -> &rcdom::Node;

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
