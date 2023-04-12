use indexmap::IndexMap;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
enum LinkTagKind {
    #[default]
    Link,
    Script,
}

/// Represents a `<link>` element.
#[derive(Default, Debug, Clone)]
pub struct LinkTag {
    attrs: IndexMap<String, String>,
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
    pub fn add(mut self, link: LinkTag) -> Self {
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
