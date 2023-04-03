use indexmap::IndexMap;
use std::fmt::Display;

/// Represents a `<script>` element.
#[derive(Default, Debug, Clone)]
pub struct ScriptTag {
    attrs: IndexMap<String, String>,
    content: Option<String>,
    // TODO: priority: Option<ScriptPriority>, BeforeInteractive, AfterInteractive?
}

impl ScriptTag {
    /// Constructs an empty `<script>` element.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets an attribute on the `<script>` element.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }

    /// Sets the inner content of the `<script>` element.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

impl Display for ScriptTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attrs = self
            .attrs
            .iter()
            .map(|(key, value)| format!("{}=\"{}\"", key, value))
            .collect::<String>();

        if let Some(content) = &self.content {
            write!(f, "<script {attrs}>{content}</script>")
        } else {
            write!(f, "<script {attrs}/>")
        }
    }
}

/// Represents a collection of `<script>` elements to include on the page.
#[derive(Default, Debug, Clone)]
pub struct PageScripts {
    tags: Vec<ScriptTag>,
}

impl PageScripts {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn iter(&self) -> std::slice::Iter<ScriptTag> {
        self.tags.iter()
    }

    pub fn add(mut self, script: ScriptTag) -> Self {
        self.tags.push(script);
        self
    }

    pub fn extend(&mut self, other: PageScripts) {
        self.tags.extend(other.tags);
    }
}
