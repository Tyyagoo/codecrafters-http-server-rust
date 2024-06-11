#![allow(dead_code)]

use std::{
    fs,
    io::{BufRead, BufReader, Read, Result, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

mod request;
mod response;
use request::{Method, Request, RequestBuilder};
use response::Response;

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

fn parse_request_line<T: Read>(
    builder: RequestBuilder,
    buf: &mut BufReader<T>,
) -> Result<RequestBuilder> {
    let mut request_line = String::new();
    let _ = buf.read_line(&mut request_line)?;
    let mut parts = request_line.split_ascii_whitespace();

    Ok(builder
        .method(parts.next().expect("Missing HTTP method."))
        .uri(parts.next().expect("Missing target."))
        .version(parts.next().expect("Missing HTTP version.")))
}

fn parse_headers<T: Read>(
    mut builder: RequestBuilder,
    buf: &mut BufReader<T>,
) -> Result<RequestBuilder> {
    loop {
        let mut line = String::new();
        let _ = buf.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }

        let (name, value) = line.split_once(':').expect("Missing colons.");
        builder = builder.header(name, value);
    }

    Ok(builder)
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut builder = Request::build();
    let mut buf = BufReader::new(&mut stream);

    builder = parse_request_line(builder, &mut buf)?;
    builder = parse_headers(builder, &mut buf)?;

    let body = match builder.peek_header("Content-Length") {
        Some(len) => {
            let size = len.parse().expect("Invalid Content-Length");
            let mut buffer = vec![0; size];
            let n = buf.read(&mut buffer)?;
            assert!(n == size);
            buffer
        }
        None => vec![],
    };

    let request = builder.body(body);

    println!("{:?}", request);
    let response = router(request);
    println!("{:?}", response);
    send_response(&mut stream, response.render().as_slice())?;

    Ok(())
}

fn router(req: Request) -> Response {
    match req.uri().as_str() {
        "/" => Response::build().body(vec![]),
        "/user-agent" => match req.header("user-agent") {
            Some(agent) => Response::build().body(agent.as_bytes().to_vec()),
            None => Response::build().status(404).body(vec![]),
        },
        _ if req.uri().starts_with("/echo") => {
            let what = req.uri().split('/').last().unwrap();
            let builder = Response::build().status(200);

            if let Some(encodings) = req.header("Accept-Encoding") {
                let available_encodings = ["gzip"];
                let encoding = encodings
                    .split(',')
                    .find(|s| available_encodings.contains(&s.trim()));

                if let Some(chosen_encoding) = encoding {
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

                        return builder
                            .header("Content-Encoding", chosen_encoding)
                            .body(compressed);
                    } else {
                        panic!()
                    }
                };
            };

            builder.body(what.as_bytes().to_vec())
        }

        _ if req.uri().starts_with("/files") => {
            let base = std::env::args().last().unwrap();
            let filename = req.uri().split("/").last().unwrap().to_string();
            let path: PathBuf = [base, filename].iter().collect();

            match req.method() {
                Method::Get => match fs::read(path) {
                    Ok(buf) => Response::build()
                        .header("Content-Type", "application/octet-stream")
                        .body(buf),
                    Err(_) => Response::build().status(404).body(vec![]),
                },
                Method::Post => {
                    fs::write(path, req.body()).unwrap();
                    Response::build().status(201).body(vec![])
                }
            }
        }

        _ => Response::build().status(404).body(vec![]),
    }
}

fn send_response(stream: &mut TcpStream, response: &[u8]) -> Result<()> {
    println!("Sending response");
    stream.write_all(response)?;
    stream.flush()?;
    Ok(())
}
