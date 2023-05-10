mod js_error;
pub use js_error::*;

mod response_error;
pub use response_error::*;

mod x {
    use crate::web::{Response, IntoResponse};

    use super::{Error, ResponseError};

    pub trait IntoResponseError {
        fn into_response_error(self) -> Response;
    }

    impl IntoResponseError for Error{
        fn into_response_error(self) -> Response {
            match self.downcast::<ResponseError>() {
                Ok(res) => (*res).into_response(),
                Err(x) => {
                    todo!()
                },
            }
        }
    }
}
