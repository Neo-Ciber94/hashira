use indexmap::IndexMap;

/// Represents the params of a route match.
#[derive(Debug, Clone, Default)]
pub struct Params {
    values: IndexMap<String, String>,
}

impl Params {
    /// Constructs an empty collection of params.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the value for the given param key, or `None` if not found.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.values.get(key.as_ref()).map(|s| s.as_str())
    }

    /// Insert the given key-value pair of params and return the previous value if any.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(key.into(), value.into())
    }

    /// Returns `true` if contains the given key
    pub fn contains(&self, key: impl AsRef<str>) -> bool {
        self.values.contains_key(key.as_ref())
    }
}

impl<'k, 'v> From<&matchit::Params<'k, 'v>> for Params {
    fn from(other: &matchit::Params) -> Self {
        let values = IndexMap::from_iter(other.iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
        Params { values }
    }
}

impl<'k, 'v> From<matchit::Params<'k, 'v>> for Params {
    fn from(other: matchit::Params) -> Self {
        let values = IndexMap::from_iter(other.iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
        Params { values }
    }
}
