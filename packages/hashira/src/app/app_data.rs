use std::ops::{Deref, DerefMut};
use http::Extensions;

/// Shared data for the application.
#[derive(Default)]
pub struct AppData(Extensions);

impl Deref for AppData {
    type Target = Extensions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AppData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
