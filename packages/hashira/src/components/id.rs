use serde::{Deserialize, Serialize};
use super::PageComponent;

/// Represents the id of a page component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PageId(pub String);

impl PageId {
    /// Returns the id for the given page.
    pub fn of<COMP>() -> Self
    where
        COMP: PageComponent,
    {
        PageId(COMP::id().to_owned())
    }
}

impl Serialize for PageId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PageId(s))
    }
}
