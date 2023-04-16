use std::{collections::BTreeMap, fmt::Display};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum LinkTagKind {
    #[default]
    Link,
    Script,
}

/// Represents a `<link>` element.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LinkTag {
    attrs: BTreeMap<String, String>,
    kind: LinkTagKind,
}

impl LinkTag {
    /// Constructs an empty `<link>` element.
    pub fn new() -> Self {
        Default::default()
    }

    /// Constructs a new `<link href='...' rel='stylesheet' type='text/css'>`.
    pub fn stylesheet(href: impl Into<String>) -> Self {
        Self::new()
            .attr("href", href)
            .attr("rel", "stylesheet")
            .attr("type", "text/css")
    }

    /// Create a empty `<script>` tag to insert on the `<head>`.
    pub fn script() -> Self {
        LinkTag {
            attrs: Default::default(),
            kind: LinkTagKind::Script,
        }
    }

    /// Sets an attribute on the `<link>` element.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }
}

impl Display for LinkTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attrs = self
            .attrs
            .iter()
            .map(|(key, value)| format!("{}=\"{}\"", key, value))
            .collect::<String>();

        match self.kind {
            LinkTagKind::Link => write!(f, "<link {attrs}/>"),
            LinkTagKind::Script => write!(f, "<script {attrs}/>"),
        }
    }
}

/// A collection of `<link>` elements.
#[derive(Default, Debug, Clone)]
pub struct PageLinks {
    tags: Vec<LinkTag>,
}

impl PageLinks {
    /// Constructs an empty `PageLinks`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns an iterator over the link elements.
    pub fn iter(&self) -> std::slice::Iter<LinkTag> {
        self.tags.iter()
    }

    /// Adds a link element.
    pub fn insert(mut self, link: LinkTag) -> Self {
        self.tags.push(link);
        self
    }

    /// Adds other page links.
    pub fn extend(&mut self, other: PageLinks) {
        self.tags.extend(other.tags);
    }
}

impl Display for PageLinks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags_html = self.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let links = tags_html.join("\n");
        write!(f, "{links}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_link_tag() {
        let link_tag = LinkTag::new();
        assert_eq!(link_tag.attrs.len(), 0);
        assert_eq!(link_tag.kind, LinkTagKind::Link);
    }

    #[test]
    fn test_new_stylesheet_link_tag() {
        let link_tag = LinkTag::stylesheet("style.css");
        assert_eq!(link_tag.attrs.len(), 3);
        assert_eq!(link_tag.attrs.get("href"), Some(&"style.css".to_string()));
        assert_eq!(link_tag.attrs.get("rel"), Some(&"stylesheet".to_string()));
        assert_eq!(link_tag.attrs.get("type"), Some(&"text/css".to_string()));
        assert_eq!(link_tag.kind, LinkTagKind::Link);
    }

    #[test]
    fn test_new_script_tag() {
        let script_tag = LinkTag::script();
        assert_eq!(script_tag.attrs.len(), 0);
        assert_eq!(script_tag.kind, LinkTagKind::Script);
    }

    #[test]
    fn test_add_attribute() {
        let link_tag = LinkTag::new().attr("href", "style.css");
        assert_eq!(link_tag.attrs.len(), 1);
        assert_eq!(link_tag.attrs.get("href"), Some(&"style.css".to_string()));
    }

    #[test]
    fn test_new_page_links() {
        let page_links = PageLinks::new();
        assert_eq!(page_links.tags.len(), 0);
    }

    #[test]
    fn test_insert_page_links() {
        let link_tag1 = LinkTag::new().attr("href", "style1.css");
        let link_tag2 = LinkTag::new().attr("href", "style2.css");
        let page_links = PageLinks::new()
            .insert(link_tag1.clone())
            .insert(link_tag2.clone());
        assert_eq!(page_links.tags.len(), 2);
        assert_eq!(page_links.tags[0], link_tag1);
        assert_eq!(page_links.tags[1], link_tag2);
    }

    #[test]
    fn test_extend_page_links() {
        let link_tag1 = LinkTag::new().attr("href", "style1.css");
        let link_tag2 = LinkTag::new().attr("href", "style2.css");
        let mut page_links1 = PageLinks::new().insert(link_tag1.clone());
        let page_links2 = PageLinks::new().insert(link_tag2.clone());
        page_links1.extend(page_links2);
        assert_eq!(page_links1.tags.len(), 2);
        assert_eq!(page_links1.tags[0], link_tag1);
        assert_eq!(page_links1.tags[1], link_tag2);
    }

    #[test]
    fn test_link_tag_display() {
        let link = LinkTag::stylesheet("style.css").attr("title", "my style");
        assert_eq!(
            link.to_string(),
            r#"<link href="style.css"rel="stylesheet"title="my style"type="text/css"/>"#
        );

        let script = LinkTag::script().attr("src", "script.js");
        assert_eq!(script.to_string(), r#"<script src="script.js"/>"#);
    }

    #[test]
    fn test_page_links_display() {
        let links = PageLinks::new();
        assert_eq!(links.to_string(), "");

        let links = PageLinks::new()
            .insert(LinkTag::stylesheet("style.css").attr("title", "my style"))
            .insert(LinkTag::script().attr("src", "script.js"));

        assert_eq!(
            links.to_string(),
            concat!(
                r#"<link href="style.css"rel="stylesheet"title="my style"type="text/css"/>"#,
                "\n",
                r#"<script src="script.js"/>"#,
            )
        );
    }
}
