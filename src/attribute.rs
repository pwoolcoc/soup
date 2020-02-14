use html5ever::rcdom::{Node, NodeData};
use crate::pattern::Pattern;

fn is_multiple(tag_name: &str, attr_name: &str) -> bool {
    match (tag_name.to_lowercase().as_str(), attr_name.to_lowercase().as_str()) {
        (_, "class")                |
        (_, "accesskey")            |
        (_, "dropzone")             |
        ("a", "rel")                |
        ("a", "rev")                |
        ("link", "rel")             |
        ("link", "rev")             |
        ("tr", "headers")           |
        ("th", "headers")           |
        ("form", "accept-charset")  |
        ("object", "archive")       |
        ("area", "rel")             |
        ("icon", "sizes")           |
        ("iframe", "sandbox")       |
        ("output", "for")           => true,
        _ => false
    }
}

fn match_list_attr<V: Pattern>(needle: &V, haystack: &str) -> bool {
    for part in haystack.split(char::is_whitespace) {
        let part = part.trim();
        if needle.matches(&part) {
            return true;
        }
    }
    false
}

pub(crate) fn list_aware_match<K: Pattern, V: Pattern>(node: &Node, attr_name: &K, attr_value: &V) -> bool {
    match node.data {
        NodeData::Element { ref name, ref attrs, ..} => {
            let attrs = attrs.borrow();
            for attr in attrs.iter() {
                let k = attr.name.local.as_ref();
                let v = attr.value.as_ref();
                if attr_name.matches(k) {
                    if is_multiple(name.local.as_ref(), &k) {
                        if match_list_attr(attr_value, &v) {
                            return true;
                        }
                    } else {
                        if attr_value.matches(v) {
                            return true;
                        }
                    }
                }
            }
        },
        _ => (),
    }
    false
}
