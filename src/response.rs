use itertools::Itertools;

#[derive(Debug)]
pub struct Response {
    status: String,
    version: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status: impl Into<String>, body: Vec<u8>) -> Response {
        Response {
            headers: Vec::from([
                ("Content-Type".into(), "text/plain".into()),
                ("Content-Length".into(), body.len().to_string()),
            ]),
            status: status.into(),
            version: "HTTP/1.1".into(),
            body,
        }
    }

    pub fn new_stream(status: impl Into<String>, body: Vec<u8>) -> Response {
        let len = body.len().to_string();
        Response {
            headers: Vec::from([
                ("Content-Type".into(), "application/octet-stream".into()),
                ("Content-Length".into(), len),
            ]),
            status: status.into(),
            version: "HTTP/1.1".into(),
            body,
        }
    }

    pub fn insert_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }

    pub fn render(&self) -> Vec<u8> {
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| format!("{}: {}\r\n", name, value))
            .join("");

        let headers_bytes = headers.as_bytes();

        [
            self.version.as_bytes(),
            b" ",
            self.status.as_bytes(),
            b"\r\n",
            headers_bytes,
            b"\r\n",
            self.body.as_slice(),
        ]
        .concat()
    }
}
