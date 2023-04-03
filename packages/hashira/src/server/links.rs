use std::fmt::Display;
use indexmap::IndexMap;

/// Represents a `<link>` element.
#[derive(Default, Debug, Clone)]
pub struct LinkTag {
    attrs: IndexMap<String, String>,
}

impl LinkTag {
    /// Constructs an empty `<link>` element.
    pub fn new() -> Self {
        Default::default()
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

        write!(f, "<link {attrs}/>")
    }
}

#[derive(Default, Debug, Clone)]
pub struct PageLinks {
    tags: Vec<LinkTag>,
}

impl PageLinks {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn iter(&self) -> std::slice::Iter<LinkTag> {
        self.tags.iter()
    }

    pub fn add(mut self, link: LinkTag) -> Self {
        self.tags.push(link);
        self
    }

    pub fn extend(&mut self, other: PageLinks) {
        self.tags.extend(other.tags);
    }
}
