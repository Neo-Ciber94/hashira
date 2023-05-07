use core::fmt;
use std::str::FromStr;

use http::Method;
use thiserror::Error;

/// Represents an HTTP method of a route as a bit field. This is a compact representation
/// of the HTTP method that allows for efficient matching of multiple methods
/// at once.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct RouteMethod(u8);

impl RouteMethod {
    /// The HTTP GET method.
    pub const GET: RouteMethod = RouteMethod(0b0001);

    /// The HTTP POST method.
    pub const POST: RouteMethod = RouteMethod(0b0010);

    /// The HTTP PUT method.
    pub const PUT: RouteMethod = RouteMethod(0b0100);

    /// The HTTP PATCH method.
    pub const PATCH: RouteMethod = RouteMethod(0b1000);

    /// The HTTP DELETE method.
    pub const DELETE: RouteMethod = RouteMethod(0b0001_0000);

    /// The HTTP HEAD method.
    pub const HEAD: RouteMethod = RouteMethod(0b0010_0000);

    /// The HTTP OPTIONS method.
    pub const OPTIONS: RouteMethod = RouteMethod(0b0100_0000);

    /// The HTTP TRACE method.
    pub const TRACE: RouteMethod = RouteMethod(0b1000_0000);

    /// Returns true if this `HttpMethod` matches the given `HttpMethod`.
    ///
    /// Matching is done by bitwise ANDing the bit fields of the two `HttpMethod`s.
    /// If the result is non-zero, the two methods match.
    pub fn matches(&self, other: &RouteMethod) -> bool {
        (self.0 & other.0) != 0
    }

    /// Returns a method that matches all.
    pub fn all() -> RouteMethod {
        RouteMethod(0b1111_1111)
    }
}

#[derive(Debug, Error)]
#[error("invalid http method: {0}")]
pub struct InvalidHttpMethod(String);

impl FromStr for RouteMethod {
    type Err = InvalidHttpMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "GET" => Ok(RouteMethod::GET),
            "POST" => Ok(RouteMethod::POST),
            "PUT" => Ok(RouteMethod::PUT),
            "PATCH" => Ok(RouteMethod::PATCH),
            "DELETE" => Ok(RouteMethod::DELETE),
            "HEAD" => Ok(RouteMethod::HEAD),
            "OPTIONS" => Ok(RouteMethod::OPTIONS),
            "TRACE" => Ok(RouteMethod::TRACE),
            _ => Err(InvalidHttpMethod(s.to_owned())),
        }
    }
}

impl std::ops::BitOr for RouteMethod {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        RouteMethod(self.0 | other.0)
    }
}

impl From<&Method> for RouteMethod {
    fn from(value: &Method) -> Self {
        match *value {
            Method::GET => RouteMethod::GET,
            Method::POST => RouteMethod::POST,
            Method::PUT => RouteMethod::PUT,
            Method::DELETE => RouteMethod::DELETE,
            Method::HEAD => RouteMethod::HEAD,
            Method::OPTIONS => RouteMethod::OPTIONS,
            Method::PATCH => RouteMethod::PATCH,
            Method::TRACE => RouteMethod::TRACE,
            _ => panic!("unsupported http method: {value}"),
        }
    }
}

impl From<Method> for RouteMethod {
    fn from(value: Method) -> Self {
        RouteMethod::from(&value)
    }
}

impl fmt::Debug for RouteMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut methods = "";
        if self.matches(&RouteMethod::GET) {
            methods = "GET";
        }
        if self.matches(&RouteMethod::POST) {
            if methods.is_empty() {
                methods = "POST";
            } else {
                write!(f, "{} | POST", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::PUT) {
            if methods.is_empty() {
                methods = "PUT";
            } else {
                write!(f, "{} | PUT", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::PATCH) {
            if methods.is_empty() {
                methods = "PATCH";
            } else {
                write!(f, "{} | PATCH", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::DELETE) {
            if methods.is_empty() {
                methods = "DELETE";
            } else {
                write!(f, "{} | DELETE", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::HEAD) {
            if methods.is_empty() {
                methods = "HEAD";
            } else {
                write!(f, "{} | HEAD", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::OPTIONS) {
            if methods.is_empty() {
                methods = "OPTIONS";
            } else {
                write!(f, "{} | OPTIONS", methods)?;
                methods = "";
            }
        }
        if self.matches(&RouteMethod::TRACE) {
            if methods.is_empty() {
                methods = "TRACE";
            } else {
                write!(f, "{} | TRACE", methods)?;
                methods = "";
            }
        }
        if !methods.is_empty() {
            write!(f, "{}", methods)?;
        }
        Ok(())
    }
}
