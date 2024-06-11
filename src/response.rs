use itertools::Itertools;

use crate::request::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct ResponseHead {
    version: String,
    status: String,
    headers: HeaderMap,
}

#[derive(Debug)]
pub struct Response {
    head: ResponseHead,
    body: Vec<u8>,
}

#[derive(Debug)]
pub struct ResponseBuilder {
    _priv: ResponseHead,
}

impl Response {
    pub fn build() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    pub fn version(&self) -> &String {
        &self.head.version
    }

    pub fn status(&self) -> &String {
        &self.head.status
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        let key = HeaderName::from_str(name).unwrap();
        self.head.headers.get(&key).map(|hv| hv.get())
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn render(&self) -> Vec<u8> {
        let headers = self
            .head
            .headers
            .iter()
            .map(|(name, value)| format!("{}: {}\r\n", name.get(), value.get()))
            .join("");

        let headers_bytes = headers.as_bytes();

        [
            self.version().as_bytes(),
            b" ",
            self.status().as_bytes(),
            b"\r\n",
            headers_bytes,
            b"\r\n",
            self.body().as_slice(),
        ]
        .concat()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        let builder = Self {
            _priv: ResponseHead {
                version: String::from("HTTP/1.1"),
                status: String::from("200 OK"),
                headers: HashMap::new(),
            },
        };

        builder.header("Content-Type", "text/plain")
    }
}

impl ResponseBuilder {
    pub fn status(mut self, code: u16) -> ResponseBuilder {
        let status = match code {
            200 => "200 OK",
            201 => "201 Created",
            404 => "404 Not Found",
            _ => unimplemented!(),
        };

        status.clone_into(&mut self._priv.status);
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> ResponseBuilder {
        use std::str::FromStr;
        let k = HeaderName::from_str(name).unwrap();
        let v = HeaderValue::from_str(value).unwrap();
        let _ = self._priv.headers.insert(k, v);
        self
    }

    pub fn body(self, body: Vec<u8>) -> Response {
        let len = body.len().to_string();
        let head = self.header("Content-Length", &len)._priv;
        Response { head, body }
    }
}
