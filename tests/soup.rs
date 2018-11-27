// TODO: any assertion commented out is a test that we won't pass yet
#[cfg(feature = "regex")]
extern crate regex;
extern crate soup;

#[cfg(feature = "regex")]
use regex::Regex;
use soup::prelude::*;

const THREE_SISTERS: &'static str = include_str!("data/three_sisters.html");

fn soup() -> Soup {
    Soup::new(THREE_SISTERS)
}

#[test]
fn simple_nav() {
    let soup = soup();
    let title = soup.tag("title").find().unwrap();
    assert_eq!(title.display(), "<title>The Dormouse's story</title>");
    assert_eq!(title.name(), "title");
    assert_eq!(title.text(), "The Dormouse's story".to_string());
    // assert_eq!(title.parent().name(), "head");
    let p = soup.tag("p").find().unwrap();
    assert_eq!(
        p.display(),
        r#"<p class="title"><b>The Dormouse's story</b></p>"#
    );
    assert_eq!(p.get("class"), Some("title".to_string()));
    let a = soup.tag("a").find().unwrap();
    assert_eq!(
        a.display(),
        r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>"#
    );
    let a_s = soup.tag("a").find_all().collect::<Vec<_>>();
    assert_eq!(
        a_s.iter()
            .map(|a| a.display())
            .collect::<Vec<_>>()
            .join("\n"),
        r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>
<a class="sister" href="http://example.com/lacie" id="link2">Lacie</a>
<a class="sister" href="http://example.com/tillie" id="link3">Tillie</a>"#
    );
}

#[test]
fn extract_all_links() {
    let soup = soup();
    let expected = [
        "http://example.com/elsie",
        "http://example.com/lacie",
        "http://example.com/tillie",
    ];
    for (i, link) in soup.tag("a").find_all().enumerate() {
        let href = link.get("href").unwrap();
        assert_eq!(href, expected[i].to_string());
    }
}

#[test]
fn extract_all_text_from_page() {
    let soup = soup();
    let text = soup.text();
    assert_eq!(
        text,
        r#"The Dormouse's story

The Dormouse's story

Once upon a time there were three little sisters; and their names were
Elsie,
Lacie and
Tillie;
and they lived at the bottom of a well.

...
"#
    );
}

#[test]
#[cfg(feature = "regex")]
fn find_with_regex() {
    let soup = soup();
    let expected = ["body", "b"];
    for (i, tag) in soup.tag(Regex::new("^b").unwrap()).find_all().enumerate() {
        assert_eq!(tag.name(), expected[i].to_string());
    }
}

#[test]
fn recursive() {
    let soup = soup();
    assert_eq!(soup.tag("title")
                    .recursive(false)
                    .find_all()
                    .count(),
                0);
}

#[test]
fn attr_with_name() {
    let soup = soup();
    let with_id = soup.attr_name("id").find_all();
    assert_eq!(
        with_id
            .map(|a| a.display())
            .collect::<Vec<_>>()
            .join("\n"),
        r#"<a class="sister" href="http://example.com/elsie" id="link1">Elsie</a>
<a class="sister" href="http://example.com/lacie" id="link2">Lacie</a>
<a class="sister" href="http://example.com/tillie" id="link3">Tillie</a>"#
    );
}

#[test]
fn multiple_value_attr() {
    let soup = Soup::new(r#"<div id="baz quux"><p class="foo bar">SOME TEXT</p></div>"#);
    let foo = soup.attr("class", "foo").find().unwrap();
    assert_eq!(foo.display(), r#"<p class="foo bar">SOME TEXT</p>"#.to_string());
    let bar = soup.attr("class", "bar").find().unwrap();
    assert_eq!(bar.display(), r#"<p class="foo bar">SOME TEXT</p>"#.to_string());
    // but a non-multiple-value attribute needs to match exactly
    let baz = soup.attr("id", "baz").find();
    assert!(baz.is_none());
}

#[test]
fn navigate_to_parent() {
    let soup = Soup::new(r#"<div id="foo"><b>FOO</b></div>"#);
    let b = soup.tag("b").find().unwrap();
    let div = b.parent().unwrap();
    assert_eq!(div.name(), "div".to_string());
}

#[test]
fn navigate_to_top_of_tree() {
    let soup = Soup::new(r#"<div id="foo"><b>FOO</b></div>"#);
    let b = soup.tag("b").find().unwrap();
    let div = b.parent().unwrap();
    let body = div.parent().unwrap();
    let html = body.parent().unwrap();
    let document = html.parent().unwrap();
    assert!(document.parent().is_none());
}

#[test]
fn child_iterator() {
    let soup = Soup::new(r#"
    <ul>
        <li>ONE</li>
        <li>TWO</li>
        <li>THREE</li>
    </ul>
    "#);
    let ul = soup.tag("ul").find().expect("Couldn't get ul");
    let children = ul.children()
        .filter(|child| child.is_element())
        .map(|child| child.text().to_string())
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 3);
    assert_eq!(children, vec!["ONE".to_string(), "TWO".to_string(), "THREE".to_string()]);
}
