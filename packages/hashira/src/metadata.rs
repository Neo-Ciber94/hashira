use std::collections::BTreeMap;

/// Represents a `<meta>` element.
#[derive(Debug, Clone)]
pub struct MetaTag {
    name: String,
    attributes: BTreeMap<String, String>,
}

impl MetaTag {
    /// Constructs a new meta tag.
    pub fn new<I>(name: impl Into<String>, attrs: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let name = name.into();
        let attributes = attrs.into_iter().collect::<BTreeMap<String, String>>();
        MetaTag { name, attributes }
    }

    pub fn with_content(name: impl Into<String>, content: impl Into<String>) -> Self {
        let name = name.into();
        let attributes = BTreeMap::from_iter([("content".to_owned(), content.into())]);
        MetaTag { name, attributes }
    }
}

/// Represents a collection of `<meta>` elements.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    // This represents the `name` and additional attributes of the <meta> tag
    tags: BTreeMap<String, MetaTag>,
}

impl Metadata {
    /// Constructs a empty collection of <meta> elements.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a `<meta name="title" content="...">` tag.
    pub fn title(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("title", content);
        self.tags.insert("title".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="description" content="...">` tag.
    pub fn description(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("description", content);
        self.tags.insert("description".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="og:type" content="...">` tag.
    pub fn og_type(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("og:type", content);
        self.tags.insert("og:type".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="og:url" content="...">` tag.
    pub fn og_url(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("og:url", content);
        self.tags.insert("og:url".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="og:title" content="...">` tag.
    pub fn og_title(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("og:title", content);
        self.tags.insert("og:title".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="og:description" content="...">` tag.
    pub fn og_description(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("og:description", content);
        self.tags.insert("og:description".to_owned(), meta);
        self
    }

    /// Adds a `<meta name="og:image" content="...">` tag.
    pub fn og_image(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("og:image", content);
        self.tags.insert("og:image".to_owned(), meta);
        self
    }
}

pub trait IntoMetaTag {
    fn into_meta_tag(self) -> MetaTag;
}

impl IntoMetaTag for (&'_ str, &'_ str) {
    fn into_meta_tag(self) -> MetaTag {
        MetaTag::with_content(self.0, self.1)
    }
}

impl IntoMetaTag for (String, String) {
    fn into_meta_tag(self) -> MetaTag {
        MetaTag::with_content(self.0, self.1)
    }
}
