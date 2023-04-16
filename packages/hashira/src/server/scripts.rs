use std::{collections::BTreeMap, fmt::Display};

/// Represents a `<script>` element to insert on the `<body>`.
/// If you want to insert a script on the head, use [`LinkTag#script`]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ScriptTag {
    attrs: BTreeMap<String, String>,
    content: Option<String>,
}

impl ScriptTag {
    /// Constructs an empty `<script>` element.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets an attribute on the `<script>` element.
    pub fn attr(mut self, key: impl Into<String>, value: impl Display) -> Self {
        self.attrs.insert(key.into(), value.to_string());
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

    pub fn insert(mut self, script: ScriptTag) -> Self {
        self.tags.push(script);
        self
    }

    pub fn extend(&mut self, other: PageScripts) {
        self.tags.extend(other.tags);
    }
}

impl Display for PageScripts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags_html = self.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let scripts = tags_html.join("\n");
        write!(f, "{scripts}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_tag_with_attributes() {
        let script_tag = ScriptTag::new()
            .attr("src", "script.js")
            .attr("async", true)
            .attr("defer", false);

        let expected = "<script async=\"true\"defer=\"false\"src=\"script.js\"/>";
        assert_eq!(script_tag.to_string(), expected);
    }

    #[test]
    fn test_script_tag_with_content() {
        let script_tag = ScriptTag::new()
            .attr("src", "script.js")
            .content("console.log('Hello, world!');");

        let expected = "<script src=\"script.js\">console.log('Hello, world!');</script>";
        assert_eq!(script_tag.to_string(), expected);
    }

    #[test]
    fn test_page_scripts_insert() {
        let mut page_scripts = PageScripts::new();

        let script_tag1 = ScriptTag::new().attr("src", "script1.js");
        let script_tag2 = ScriptTag::new().attr("src", "script2.js");

        page_scripts = page_scripts.insert(script_tag1.clone());
        page_scripts = page_scripts.insert(script_tag2.clone());

        assert_eq!(page_scripts.iter().count(), 2);
        assert_eq!(page_scripts.iter().next().unwrap(), &script_tag1);
        assert_eq!(page_scripts.iter().nth(1).unwrap(), &script_tag2);
    }

    #[test]
    fn test_page_scripts_extend() {
        let mut page_scripts1 = PageScripts::new();

        let script_tag1 = ScriptTag::new().attr("src", "script1.js");
        let script_tag2 = ScriptTag::new().attr("src", "script2.js");

        page_scripts1 = page_scripts1.insert(script_tag1.clone());
        page_scripts1 = page_scripts1.insert(script_tag2.clone());

        let mut page_scripts2 = PageScripts::new();

        let script_tag3 = ScriptTag::new().attr("src", "script3.js");
        let script_tag4 = ScriptTag::new().attr("src", "script4.js");

        page_scripts2 = page_scripts2.insert(script_tag3.clone());
        page_scripts2 = page_scripts2.insert(script_tag4.clone());

        page_scripts1.extend(page_scripts2);

        assert_eq!(page_scripts1.iter().count(), 4);
        assert_eq!(page_scripts1.iter().next().unwrap(), &script_tag1);
        assert_eq!(page_scripts1.iter().nth(1).unwrap(), &script_tag2);
        assert_eq!(page_scripts1.iter().nth(2).unwrap(), &script_tag3);
        assert_eq!(page_scripts1.iter().nth(3).unwrap(), &script_tag4);
    }
}
