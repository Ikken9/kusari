use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod client;

#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: u16,
    pub status: http::StatusCode,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Error {
    message: String,
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
