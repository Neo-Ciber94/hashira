use std::{collections::BTreeMap, fmt::Display};

/// Represents a `<meta>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTag {
    name: String,
    attrs: BTreeMap<String, String>,
}

impl MetaTag {
    /// Constructs a new meta tag.
    pub fn new<I>(name: impl Into<String>, attrs: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let name = name.into();
        let attrs = attrs.into_iter().collect::<BTreeMap<String, String>>();
        MetaTag { name, attrs }
    }

    /// Constructs a tag in the form: `<meta name='...' content='...' />`
    pub fn with_content(name: impl Into<String>, content: impl Into<String>) -> Self {
        let name = name.into();
        let attrs = BTreeMap::from_iter([("content".to_owned(), content.into())]);
        MetaTag { name, attrs }
    }

    /// Returns the value of the `name` attribute.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the attributes of the tag.
    pub fn attrs(&self) -> std::collections::btree_map::Iter<String, String> {
        self.attrs.iter()
    }
}

impl Display for MetaTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("name=\"{}\"", self.name);
        let attrs = self
            .attrs
            .iter()
            .map(|(key, value)| format!("{}=\"{}\"", key, value))
            .collect::<String>();

        write!(f, "<meta {name} {attrs}/>")
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

    /// Returns an iterator over the meta elements.
    pub fn meta_tags(&self) -> std::collections::btree_map::Values<String, MetaTag> {
        self.tags.values()
    }

    /// Adds a `<meta name="viewport" content="...">` tag.
    pub fn viewport(mut self, content: impl Into<String>) -> Self {
        let meta = MetaTag::with_content("viewport", content);
        self.tags.insert("viewport".to_owned(), meta);
        self
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

    /// Merge all the meta tags with the other meta tags.
    pub fn extend(&mut self, other: Metadata) {
        self.tags.extend(other.tags);
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags_html = self
            .meta_tags()
            .map(|meta| meta.to_string())
            .collect::<Vec<_>>();

        let meta = tags_html.join("\n");
        write!(f, "{meta}")
    }
}

/// Converts an element to a `MetaTag`.
pub trait IntoMetaTag {
    /// Returns a `MetaTag` from this element.
    fn into_meta_tag(self) -> MetaTag;
}

impl IntoMetaTag for MetaTag {
    fn into_meta_tag(self) -> MetaTag {
        self
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_meta_tag() {
        let meta_tag = MetaTag::new("test", vec![("key".to_owned(), "value".to_owned())]);
        assert_eq!(meta_tag.name(), "test");
        assert_eq!(
            meta_tag.attrs().next().unwrap(),
            (&"key".to_owned(), &"value".to_owned())
        );
    }

    #[test]
    fn test_with_content_meta_tag() {
        let meta_tag = MetaTag::with_content("test", "content");
        assert_eq!(meta_tag.name(), "test");
        assert_eq!(
            meta_tag.attrs().next().unwrap(),
            (&"content".to_owned(), &"content".to_owned())
        );
    }

    #[test]
    fn test_display_meta_tag() {
        let meta_tag = MetaTag::with_content("test", "content");
        let display = format!("{}", meta_tag);
        assert_eq!(display, "<meta name=\"test\" content=\"content\"/>");
    }

    #[test]
    fn test_metadata() {
        let metadata = Metadata::new()
            .viewport("width=device-width, initial-scale=1")
            .title("Example")
            .og_type("website")
            .og_url("https://example.com")
            .og_title("Example Title")
            .og_description("Example Description")
            .og_image("https://example.com/image.png");

        let mut expected_tags = BTreeMap::new();
        expected_tags.insert(
            "viewport".to_owned(),
            MetaTag::with_content("viewport", "width=device-width, initial-scale=1"),
        );
        expected_tags.insert(
            "title".to_owned(),
            MetaTag::with_content("title", "Example"),
        );
        expected_tags.insert(
            "og:type".to_owned(),
            MetaTag::with_content("og:type", "website"),
        );
        expected_tags.insert(
            "og:url".to_owned(),
            MetaTag::with_content("og:url", "https://example.com"),
        );
        expected_tags.insert(
            "og:title".to_owned(),
            MetaTag::with_content("og:title", "Example Title"),
        );
        expected_tags.insert(
            "og:description".to_owned(),
            MetaTag::with_content("og:description", "Example Description"),
        );
        expected_tags.insert(
            "og:image".to_owned(),
            MetaTag::with_content("og:image", "https://example.com/image.png"),
        );

        assert_eq!(
            metadata.meta_tags().collect::<Vec<&MetaTag>>(),
            expected_tags.values().collect::<Vec<&MetaTag>>()
        );
    }

    #[test]
    fn test_extend() {
        let mut meta1 = Metadata::new();
        meta1 = meta1.title("Hello").description("World");

        let mut meta2 = Metadata::new();
        meta2 = meta2.viewport("width=device-width").og_type("website");

        meta1.extend(meta2);

        assert_eq!(meta1.meta_tags().count(), 4);
        assert!(meta1.meta_tags().any(|m| m.name() == "viewport"));
        assert!(meta1.meta_tags().any(|m| m.name() == "title"));
        assert!(meta1.meta_tags().any(|m| m.name() == "description"));
        assert!(meta1.meta_tags().any(|m| m.name() == "og:type"));
    }
}
