use std::ops::Deref;

use yew::{Children, Properties};

/// Represents a the properties of a component with a children.
#[derive(PartialEq, Properties, Debug, Default, Clone)]
pub struct PropsWithChildren<T: PartialEq> {
    /// The data.
    pub data: T,

    /// The children
    pub children: Children,
}

impl<T: PartialEq + Clone> PropsWithChildren<T> {
    /// Clones the inner data.
    pub fn cloned(&self) -> T {
        self.data.clone()
    }
}

impl<T: PartialEq> Deref for PropsWithChildren<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
