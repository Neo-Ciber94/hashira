use indexmap::IndexMap;

/// Represents the params of a route match.
#[derive(Debug, Clone, Default)]
pub struct Params {
    map: IndexMap<String, String>,
}

impl Params {
    /// Returns the value for the given param key, or `None` if not found.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.map.get(key.as_ref()).map(|s| s.as_str())
    }

    /// Returns the key-value at the given index.
    pub fn get_index(&self, index: usize) -> Option<(&str, &str)> {
        self.map
            .get_index(index)
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Returns `true` if contains the given key
    pub fn contains(&self, key: impl AsRef<str>) -> bool {
        self.map.contains_key(key.as_ref())
    }

    /// Returns the number of key-values.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if there are no values.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over the key-values.
    pub fn iter(&self) -> indexmap::map::Iter<String, String> {
        self.map.iter()
    }
}

impl FromIterator<(String, String)> for Params {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let map = IndexMap::from_iter(iter);
        Params { map }
    }
}

impl<'k, 'v> FromIterator<(&'k str, &'v str)> for Params {
    fn from_iter<T: IntoIterator<Item = (&'k str, &'v str)>>(iter: T) -> Self {
        let iter = iter.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned()));
        Params::from_iter(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params() {
        let mut params = Params::default();
        params.map.insert("foo".to_owned(), "bar".to_owned());
        params.map.insert("baz".to_owned(), "qux".to_owned());

        // Test get method
        assert_eq!(params.get("foo"), Some("bar"));
        assert_eq!(params.get("nonexistent"), None);

        // Test get_index method
        assert_eq!(params.get_index(0), Some(("foo", "bar")));
        assert_eq!(params.get_index(1), Some(("baz", "qux")));
        assert_eq!(params.get_index(2), None);

        // Test contains method
        assert!(params.contains("foo"));
        assert!(!params.contains("nonexistent"));

        // Test len method
        assert_eq!(params.len(), 2);

        // Test iter method
        let mut iter = params.iter();
        assert_eq!(iter.next(), Some((&"foo".to_owned(), &"bar".to_owned())));
        assert_eq!(iter.next(), Some((&"baz".to_owned(), &"qux".to_owned())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_from_iterator() {
        let params = vec![("foo", "bar"), ("baz", "qux")]
            .into_iter()
            .collect::<Params>();

        assert_eq!(params.get("foo"), Some("bar"));
        assert_eq!(params.get("baz"), Some("qux"));
        assert_eq!(params.len(), 2);
    }
}
