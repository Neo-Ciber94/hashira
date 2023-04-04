use super::layout_data::PageLayoutData;
use std::ops::{Deref, DerefMut};

pub struct LayoutContext {
    layout_data: PageLayoutData,
}

impl LayoutContext {
    pub fn new(layout_data: PageLayoutData) -> Self {
        LayoutContext { layout_data }
    }
}

impl Deref for LayoutContext {
    type Target = PageLayoutData;

    fn deref(&self) -> &Self::Target {
        &self.layout_data
    }
}

impl DerefMut for LayoutContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.layout_data
    }
}
