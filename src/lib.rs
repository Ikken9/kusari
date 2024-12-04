use std::fmt;
use std::fmt::{Display, Formatter};
use http::{HeaderMap, Method};
use url::Url;

pub mod client;

#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Url,
    pub version: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: u16,
    pub status: http::StatusCode,
    pub reason_phrase: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: Default::default(),
            uri: Url::parse("http://localhost").unwrap(),
            version: "".to_string(),
            headers: Default::default(),
            body: vec![],
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\n{:?}{}",
            self.method,
            self.uri,
            self.version,
            self.headers,
            if !self.body.is_empty() {
                format!(
                    "\nBody:\n{}",
                    String::from_utf8_lossy(&self.body)
                )
            } else {
                String::new()
            }
        )
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}\n{}\n{:?}{}",
            self.status_code,
            self.reason_phrase,
            self.status,
            self.headers,
            if !self.body.is_empty() {
                format!(
                    "\nBody:\n{}",
                    String::from_utf8_lossy(&self.body)
                )
            } else {
                String::new()
            }
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {

}
