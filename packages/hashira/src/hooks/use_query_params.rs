use crate::web::QueryParamsError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::ops::Deref;
use yew::{hook, use_state};

// Define a struct QueryParamsHandle with a generic type Q that has
// a single field query_params of type Q
#[derive(Serialize, Deserialize)]
pub struct QueryParamsHandle<Q> {
    query_params: Q,
}

impl<Q> QueryParamsHandle<Q>
where
    Q: Serialize + DeserializeOwned,
{
    // Define a new() method that creates a new QueryParamsHandle instance
    // and initializes its query_params field by parsing the search string
    // from the current window's location using serde_qs
    pub fn new() -> Result<Self, QueryParamsError> {
        let window = web_sys::window().expect("failed to get window");
        let location = window.location();
        let search = location.search().expect("failed to get location.search");

        // Return an error if the search string is empty
        if search.is_empty() {
            return Err(QueryParamsError::NotFound);
        }

        // Deserialize the search string using serde_qs and return any errors
        serde_qs::from_str(search.as_str()).map_err(QueryParamsError::Parse)
    }

    // Define a set() method that updates the query_params field with
    // a new value and sets the search string in the current window's location
    // to the serialized value of the updated query_params field
    pub fn set(&mut self, query_params: Q) {
        self.query_params = query_params;
        let window = web_sys::window().unwrap();
        let location = window.location();
        location
            .set_search(&serde_qs::to_string(&self.query_params).unwrap())
            .unwrap();
    }

    // Define a set_without_refresh() method that updates the query_params field
    // with a new value and pushes a new state to the window's history with
    // the serialized value of the updated query_params field as the search string
    pub fn set_without_refresh(&mut self, query_params: Q) {
        self.query_params = query_params;
        let window = web_sys::window().unwrap();
        let location = window.location();
        let history = window.history().unwrap();
        history
            .push_state_with_url(
                &location,
                "",
                Some(&serde_qs::to_string(&self.query_params).unwrap()),
            )
            .unwrap();
    }
}

// Implement the Deref trait for QueryParamsHandle so that instances of
// QueryParamsHandle can be dereferenced to their query_params field
impl<Q> Deref for QueryParamsHandle<Q> {
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        &self.query_params
    }
}

// Returns a handle to this route query params parsed as the given type.
#[hook]
pub fn use_query_params<Q: Serialize + DeserializeOwned>(
) -> Result<QueryParamsHandle<Q>, QueryParamsError> {
    todo!()
}
