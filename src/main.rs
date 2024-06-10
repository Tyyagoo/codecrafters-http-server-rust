#![allow(dead_code)]

use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader, Error, Read, Result, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use response::Response;

mod response;

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug)]
struct Request {
    method: HttpMethod,
    target: String,
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                // TODO: use tokio
                std::thread::spawn(move || {
                    handle_connection(stream).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn parse_request_line<T: Read>(buf: &mut BufReader<T>) -> Result<(HttpMethod, String, String)> {
    let mut request_line = String::new();
    let _ = buf.read_line(&mut request_line)?;
    let mut parts = request_line.split_ascii_whitespace();

    let method = match parts.next().expect("Missing HTTP method.") {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        _ => unimplemented!(),
    };

    let target = parts.next().expect("Missing target.");
    let version = parts.next().expect("Missing HTTP version.");

    Ok((method, target.into(), version.into()))
}

fn parse_headers<T: Read>(buf: &mut BufReader<T>) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();

    loop {
        let mut line = String::new();
        let _ = buf.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }

        let (name, value) = line.split_once(':').expect("Missing colons.");
        headers.insert(name.into(), value.trim().into());
    }

    Ok(headers)
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = BufReader::new(&mut stream);

    let (method, target, version) = parse_request_line(&mut buf).unwrap();
    let headers = parse_headers(&mut buf)?;

    let body = match headers.get("Content-Length") {
        Some(len) => {
            let size = len.parse().expect("Invalid Content-Length");
            let mut buffer = vec![0; size];
            let n = buf.read(&mut buffer)?;
            assert!(n == size);
            buffer
        }
        None => vec![],
    };

    let request = Request {
        method,
        target,
        version,
        headers,
        body,
    };

    println!("{:?}", request);
    let response = router(request);
    println!("{:?}", response);
    send_response(&mut stream, response.render().as_slice())?;

    Ok(())
}

fn router(req: Request) -> Response {
    match req.target.as_str() {
        "/" => Response::new("200 OK", vec![]),
        "/user-agent" => {
            let agent = req.headers.get("User-Agent").unwrap();
            Response::new("200 OK", agent.as_bytes().to_vec())
        }
        _ if req.target.starts_with("/echo") => {
            let what = req.target.split('/').last().unwrap();

            match req.headers.get("Accept-Encoding") {
                Some(encodings) => {
                    let available_encodings = ["gzip"];
                    let encoding = encodings
                        .split(',')
                        .find(|s| available_encodings.contains(&s.trim()));

                    match encoding {
                        Some(e) => {
                            // hmm.
                            let mut command = std::process::Command::new("gzip")
                                .arg("-c")
                                .stdin(std::process::Stdio::piped())
                                .stdout(std::process::Stdio::piped())
                                .spawn()
                                .unwrap();

                            {
                                let stdin = command.stdin.as_mut().unwrap();
                                stdin.write_all(what.as_bytes()).unwrap();
                            }

                            let output = command.wait_with_output().unwrap();
                            if output.status.success() {
                                let compressed = output.stdout;

                                let mut res = Response::new("200 OK", compressed);
                                res.insert_header("Content-Encoding", e);
                                res
                            } else {
                                panic!()
                            }
                        }

                        _ => Response::new("200 OK", what.as_bytes().to_vec()),
                    }
                }

                None => Response::new("200 OK", what.as_bytes().to_vec()),
            }
        }

        _ if req.target.starts_with("/files") => {
            let base = std::env::args().last().unwrap();
            let filename = req.target.split("/").last().unwrap().to_string();
            let path: PathBuf = [base, filename].iter().collect();

            match req.method {
                HttpMethod::Get => match fs::read(path) {
                    Ok(buf) => Response::new_stream("200 OK", buf),
                    Err(_) => Response::new("404 Not Found", vec![]),
                },
                HttpMethod::Post => {
                    fs::write(path, req.body).unwrap();
                    Response::new("201 Created", vec![])
                }
                _ => panic!(),
            }
        }

        _ => Response::new("404 Not Found", vec![]),
    }
}

fn send_response(stream: &mut TcpStream, response: &[u8]) -> Result<()> {
    println!("Sending response");
    stream.write_all(response)?;
    stream.flush()?;
    Ok(())
}
