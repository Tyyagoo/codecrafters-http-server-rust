use itertools::Itertools;

#[derive(Debug)]
pub struct Response {
    status: String,
    version: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Response {
    pub fn new(status: impl Into<String>, body: impl Into<String>) -> Response {
        Response {
            status: status.into(),
            body: body.into(),
            version: "HTTP/1.1".into(),
            headers: Vec::new(),
        }
    }

    pub fn insert_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }

    pub fn render(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| format!("{}: {}\r\n", name, value))
            .join("");

        format!(
            "{} {}\r\n{}\r\n{}",
            self.version, self.status, headers, self.body
        )
    }
}
