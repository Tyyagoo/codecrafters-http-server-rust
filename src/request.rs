use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct HeaderName(String);

#[derive(Debug)]
pub struct HeaderValue(String); // WARN: can contain non-utf8 values

pub type HeaderMap = HashMap<HeaderName, HeaderValue>;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
struct RequestHead {
    method: Method,
    uri: String,
    version: String,
    headers: HeaderMap,
}

#[derive(Debug)]
pub struct Request {
    head: RequestHead,
    body: Vec<u8>,
}

pub struct RequestBuilder {
    _priv: RequestHead,
}

impl Request {
    pub fn build() -> RequestBuilder {
        RequestBuilder::default()
    }

    pub fn method(&self) -> &Method {
        &self.head.method
    }

    pub fn uri(&self) -> &String {
        &self.head.uri
    }

    pub fn version(&self) -> &String {
        &self.head.version
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        let key = HeaderName::from_str(name).unwrap();
        self.head.headers.get(&key).map(|hv| &hv.0)
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }
}

impl Default for RequestBuilder {
    fn default() -> RequestBuilder {
        RequestBuilder {
            _priv: RequestHead {
                method: Method::Get,
                uri: String::new(),
                version: String::from("HTTP/1.1"),
                headers: HashMap::new(),
            },
        }
    }
}

impl RequestBuilder {
    pub fn method(mut self, method: &str) -> RequestBuilder {
        let m = match method {
            "GET" | "get" => Method::Get,
            "POST" | "post" => Method::Post,
            _ => unimplemented!(),
        };
        self._priv.method = m;
        self
    }

    pub fn uri(mut self, uri: &str) -> RequestBuilder {
        uri.clone_into(&mut self._priv.uri);
        self
    }

    pub fn version(mut self, version: &str) -> RequestBuilder {
        version.clone_into(&mut self._priv.version);
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> RequestBuilder {
        let k = HeaderName::from_str(name).unwrap();
        let v = HeaderValue::from_str(value).unwrap();
        let _ = self._priv.headers.insert(k, v);
        self
    }

    pub fn body(self, content: Vec<u8>) -> Request {
        Request {
            head: self._priv,
            body: content,
        }
    }

    pub fn peek_header(&self, name: &str) -> Option<&String> {
        let key = HeaderName::from_str(name).unwrap();
        self._priv.headers.get(&key).map(|hv| &hv.0)
    }
}

impl HeaderName {
    pub fn get(&self) -> &String {
        &self.0
    }
}

impl HeaderValue {
    pub fn get(&self) -> &String {
        &self.0
    }
}

#[derive(Debug)]
pub struct ConversionError;

impl std::str::FromStr for HeaderName {
    type Err = ConversionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(HeaderName(value.to_ascii_lowercase()))
    }
}

impl std::str::FromStr for HeaderValue {
    type Err = ConversionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(HeaderValue(value.trim().to_owned()))
    }
}
