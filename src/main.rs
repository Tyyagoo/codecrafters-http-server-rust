use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Error, Read, Result, Write},
    net::{TcpListener, TcpStream},
};

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
            Ok(mut stream) => {
                println!("accepted new connection");
                handle_connection(stream)?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = BufReader::new(&mut stream);
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
    println!("Method: {:?}", method);

    let target = start_line.next().expect("Missing target.");
    println!("Target: {}", target);

    let version = start_line.next().expect("Missing HTTP version.");
    println!("Version: {}", version);

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
    println!("Headers: {:?}", headers);

    let request = Request {
        method,
        target,
        version,
        headers,
        body: "",
    };
    let response = router(request);
    send_response(&mut stream, response)?;

    Ok(())
}

fn router(req: Request) -> &[u8] {
    match req.target {
        "/" => b"HTTP/1.1 200 OK\r\n\r\n",
        _ => b"HTTP/1.1 404 Not Found\r\n\r\n",
    }
}

fn send_response(stream: &mut TcpStream, response: &[u8]) -> Result<()> {
    println!("Sending response");
    stream.write_all(response)?;
    stream.flush()?;
    Ok(())
}
