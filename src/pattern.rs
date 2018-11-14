//! Traits & impls for matching types with strings

#[cfg(feature = "regex")]
use regex::Regex;

/// A trait used to indicate a type which can be used to match a value
///
/// Any type that implements this trait can be passed to the various
/// QueryBuilder methods in order to match an element
///
/// # Example
///
/// ```rust
/// # extern crate soup;
/// use soup::{pattern::Pattern, prelude::*};
///
/// #[derive(Clone)]
/// struct MyType(String);
///
/// impl Pattern for MyType {
///     fn matches(&self, haystack: &str) -> bool {
///         self.0.matches(haystack)
///     }
/// }
///
/// let soup = Soup::new(r#"<div id="foo"></div>"#);
/// let result = soup.tag(MyType("div".to_string())).find().unwrap();
/// assert_eq!(result.get("id").unwrap(), "foo".to_string());
/// ```
pub trait Pattern: Clone {
    /// Matches the `Pattern` with the value `haystack`
    fn matches(&self, haystack: &str) -> bool;
}

impl<'a, P> Pattern for &'a P
where P: Pattern
{
    fn matches(&self, haystack: &str) -> bool {
        (*self).matches(haystack)
    }
}

impl Pattern for bool {
    fn matches(&self, _haystack: &str) -> bool {
        *self
    }
}

impl<'a> Pattern for &'a str {
    fn matches(&self, haystack: &str) -> bool {
        *self == haystack
    }
}

impl Pattern for String {
    fn matches(&self, haystack: &str) -> bool {
        self == haystack
    }
}

#[cfg(feature = "regex")]
impl Pattern for Regex {
    fn matches(&self, haystack: &str) -> bool {
        self.is_match(haystack)
    }
}
