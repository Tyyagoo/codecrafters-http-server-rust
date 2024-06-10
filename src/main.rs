#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Error, Read, Result, Write},
    net::{TcpListener, TcpStream},
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
struct Request<'a> {
    method: HttpMethod,
    target: &'a str,
    version: &'a str,
    headers: HashMap<String, &'a str>,
    body: &'a str,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
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

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let buf = BufReader::new(&mut stream);
    let lines: Vec<String> = buf
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut start_line = lines[0].split_ascii_whitespace();

    let method = match start_line.next().expect("Missing HTTP method.") {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        _ => unimplemented!(),
    };

    let target = start_line.next().expect("Missing target.");
    let version = start_line.next().expect("Missing HTTP version.");

    let mut headers = HashMap::new();
    for line in lines.iter() {
        let line = line.trim();
        if line.is_empty() {
            break;
        }

        let mut parts = line.split_ascii_whitespace();
        headers.insert(
            parts.next().unwrap().replace(':', ""),
            parts.next().unwrap(),
        );
    }

    let request = Request {
        method,
        target,
        version,
        headers,
        body: "",
    };

    println!("{:?}", request);
    let response = router(request);
    send_response(&mut stream, response.render().as_bytes())?;

    Ok(())
}

fn router(req: Request) -> Response {
    match req.target {
        "/" => Response::new("200 OK", ""),
        "/user-agent" => {
            let agent = req.headers.get("User-Agent").unwrap();
            Response::new("200 OK", *agent)
        }
        _ if req.target.starts_with("/echo") => {
            let what = req.target.split('/').last().unwrap();
            Response::new("200 OK", what)
        }
        _ => Response::new("404 Not Found", ""),
    }
}

fn send_response(stream: &mut TcpStream, response: &[u8]) -> Result<()> {
    println!("Sending response");
    stream.write_all(response)?;
    stream.flush()?;
    Ok(())
}
