use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};
use get_if_addrs::{get_if_addrs, IfAddr};
use threadpool::ThreadPool;

fn main() {
    let ip = get_local_ip().unwrap();
    let listener: TcpListener = TcpListener::bind(ip.clone()+":0").unwrap();
    let pool = ThreadPool::new(4);

    let server_addrs = listener.local_addr().unwrap();
    println!("Server is running on http://{server_addrs}");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("Request: {request_line}");

    // Parse the path from the request line
    let path = parse_path(&request_line);

    // Match the request path to appropriate file or endpoint
    let (status_line, file_path) = if path == "/" {
        ("HTTP/1.1 200 OK", &"static/index.html".to_string())
    }else {
        // Try to locate the file within the static directory
        let full_path = format!("static{}", path);
        if Path::new(&full_path.clone()).exists() {
            ("HTTP/1.1 200 OK", &full_path.clone())
        } else {
            println!("file not found: {full_path}");
            ("HTTP/1.1 404 NOT FOUND", &"static/404.html".to_string())
        }
    };

    send_response(&mut stream, status_line, file_path);
}

/// Parse the requested path from the HTTP request line.
fn parse_path(request_line: &str) -> &str {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() > 1 {
        parts[1] // The path part of the request
    } else {
        "/"
    }
}

/// Send an HTTP response with the given status and file path.
fn send_response(stream: &mut TcpStream, status_line: &str, file_path: &str) {
    // Read file contents
    let contents = fs::read(file_path).unwrap_or_else(|_| Vec::new());

    let length = contents.len();

    // Build and send response
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();

    println!("Response: {status_line}, File: {file_path}");
}

fn get_local_ip() -> Option<String> {
    let if_addrs = get_if_addrs().ok()?;

    for iface in if_addrs {
        if let IfAddr::V4(v4_addr) = iface.addr {
            if !v4_addr.ip.is_loopback() {
                return Some(v4_addr.ip.to_string());
            }
        }
    }

    None
}