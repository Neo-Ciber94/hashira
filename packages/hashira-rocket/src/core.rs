use hashira::{
    app::AppService,
    web::{Body, Response},
};
use rocket::{
    data::FromData,
    fs::FileServer,
    futures::TryStreamExt,
    http::Method::*,
    outcome,
    request::FromRequest,
    route::{self, Handler},
    State,
};
use rocket::{Build, Rocket};

#[doc(hidden)]
pub struct RequestWithoutBody(hashira::web::Request<()>);

/// A responder for handling a rocket request.
#[derive(Clone)]
pub struct DefaultRequestHandler;

impl DefaultRequestHandler {
    /// Returns all the routes this handler will handle.
    pub fn routes(rank: Option<isize>) -> Vec<rocket::Route> {
        let mut routes = vec![];
        for method in [Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch] {
            routes.push(rocket::Route::ranked(
                rank,
                method,
                "/<path..>",
                DefaultRequestHandler,
            ));
        }

        routes
    }
}

impl Into<Vec<rocket::Route>> for DefaultRequestHandler {
    fn into(self) -> Vec<rocket::Route> {
        DefaultRequestHandler::routes(Some(100))
    }
}

#[rocket::async_trait]
impl Handler for DefaultRequestHandler {
    async fn handle<'r>(
        &self,
        rocket_req: &'r rocket::Request<'_>,
        data: rocket::data::Data<'r>,
    ) -> rocket::route::Outcome<'r> {
        let req = match RequestWithoutBody::from_request(rocket_req).await {
            outcome::Outcome::Success(req) => req,
            outcome::Outcome::Failure((status, _)) => return route::Outcome::Failure(status),
            outcome::Outcome::Forward(_) => return route::Outcome::Forward(data),
        };

        let service: &State<AppService> = match FromRequest::from_request(rocket_req).await {
            outcome::Outcome::Success(s) => s,
            outcome::Outcome::Failure((status, _)) => return route::Outcome::Failure(status),
            outcome::Outcome::Forward(_) => return route::Outcome::Forward(data),
        };

        let bytes = match Vec::<u8>::from_data(rocket_req, data).await {
            outcome::Outcome::Success(b) => b,
            outcome::Outcome::Failure((status, _)) => return route::Outcome::Failure(status),
            outcome::Outcome::Forward(data) => return route::Outcome::Forward(data),
        };

        let req = req.0.map(|_| Body::from(bytes));
        let res = service.handle(req).await;

        let rocket_res = map_response(res).await;
        route::Outcome::Success(rocket_res)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestWithoutBody {
    type Error = hashira::web::Error;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        fn method_from_rocket(req: &rocket::Request) -> hashira::web::method::Method {
            match req.method() {
                Get => hashira::web::method::Method::GET,
                Put => hashira::web::method::Method::PUT,
                Post => hashira::web::method::Method::POST,
                Delete => hashira::web::method::Method::DELETE,
                Options => hashira::web::method::Method::OPTIONS,
                Head => hashira::web::method::Method::HEAD,
                Trace => hashira::web::method::Method::TRACE,
                Connect => hashira::web::method::Method::CONNECT,
                Patch => hashira::web::method::Method::PATCH,
            }
        }

        let mut builder = hashira::web::Request::builder()
            .method(method_from_rocket(req))
            .uri(req.uri().to_string());

        for header in req.headers().iter() {
            builder = builder.header(header.name.as_str(), header.value.into_owned());
        }

        let req = match builder.body(()) {
            Ok(x) => x,
            Err(err) => {
                return rocket::request::Outcome::Failure((
                    rocket::http::Status::InternalServerError,
                    err,
                ))
            }
        };
        rocket::request::Outcome::Success(RequestWithoutBody(req))
    }
}

// Returns a function to attach the hashira router to `Rocket`.
pub fn router(app_service: AppService) -> impl FnOnce(Rocket<Build>) -> Rocket<Build> {
    let static_dir = hashira::env::get_static_dir();
    let serve_dir = get_current_dir().join("public");

    move |rocket| {
        rocket
            .manage(app_service)
            .mount(&static_dir, FileServer::from(serve_dir))
            .mount("/", DefaultRequestHandler)
    }
}

async fn map_response(res: Response) -> rocket::Response<'static> {
    let mut builder = rocket::Response::build();

    // Set the status code
    let status =
        rocket::http::Status::from_code(res.status().as_u16()).expect("invalid status code");
    builder.status(status);

    // Set the headers
    for (name, value) in res.headers() {
        let v = value.to_str().unwrap().to_string();
        builder.header_adjoin(rocket::http::Header::new(name.to_string(), v));
    }

    // Set the body
    match res.into_body().into_inner() {
        hashira::web::BodyInner::Bytes(bytes) => {
            let len = bytes.len();
            let buf = std::io::Cursor::new(bytes);
            builder.sized_body(len, buf);
        }
        hashira::web::BodyInner::Stream(stream) => {
            let s = stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let reader = tokio_util::io::StreamReader::new(s);
            let body = rocket::response::stream::ReaderStream::one(reader);
            builder.streamed_body(body);
        }
    }

    builder.finalize()
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
