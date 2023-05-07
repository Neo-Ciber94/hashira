use crate::{components::AnyComponent, app::ErrorPageHandler};
use http::StatusCode;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("failed to insert error page for `{0}`, it already exists")]
pub struct ErrorPageConflictError(StatusCode);

/// Contains the error routes for the client.
#[derive(Default, Clone, PartialEq)]
pub struct ErrorRouter {
    routes: HashMap<StatusCode, AnyComponent<serde_json::Value>>,
    fallback: Option<AnyComponent<serde_json::Value>>,
}

impl ErrorRouter {
    /// Constructs a new error router.
    pub fn new() -> Self {
        ErrorRouter {
            routes: HashMap::new(),
            fallback: None,
        }
    }

    /// Adds a component for the given `StatusCode`.
    pub fn insert(
        &mut self,
        status: StatusCode,
        component: AnyComponent<serde_json::Value>,
    ) -> Result<(), ErrorPageConflictError> {
        if self.routes.insert(status, component).is_some() {
            return Err(ErrorPageConflictError(status));
        }

        Ok(())
    }

    /// Adds a handler for any status code.
    pub fn fallback(&mut self, component: AnyComponent<serde_json::Value>) {
        self.fallback = Some(component);
    }

    /// Returns the component to render for the given `StatusCode`.
    pub fn find(&self, status: &StatusCode) -> Option<&AnyComponent<serde_json::Value>> {
        self.routes.get(status).or(self.fallback.as_ref())
    }
}

/// Contains the error routes for the client.
#[derive(Default)]
pub struct ServerErrorRouter {
    routes: HashMap<StatusCode, ErrorPageHandler>,
    fallback: Option<ErrorPageHandler>,
}

impl ServerErrorRouter {
    /// Constructs a new error router.
    pub fn new() -> Self {
        ServerErrorRouter {
            routes: HashMap::new(),
            fallback: None,
        }
    }

    /// Adds a handler for the given `StatusCode`.
    pub fn insert(
        &mut self,
        status: StatusCode,
        handler: ErrorPageHandler,
    ) -> Result<(), ErrorPageConflictError> {
        if self.routes.insert(status, handler).is_some() {
            return Err(ErrorPageConflictError(status));
        }

        Ok(())
    }

    /// Adds a component to handle for error status code.
    pub fn fallback(&mut self, handler: ErrorPageHandler) {
        self.fallback = Some(handler);
    }

    /// Returns the handler for the given `StatusCode`.
    pub fn find(&self, status: &StatusCode) -> Option<&ErrorPageHandler> {
        match self.routes.get(status) {
            Some(handler) => Some(handler),
            None => self.fallback.as_ref(),
        }
    }
}
