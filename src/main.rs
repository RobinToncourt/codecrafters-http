#[allow(unused_imports)]
use std::fmt;
use std::io::{
    prelude::*,
    Write,
    BufReader,
};
use std::net::{
    TcpStream,
    TcpListener,
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum HttpError {

}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct HttpRequest {
    request_line: RequestLine,
    headers: Vec<Header>,
    body: String,
}

impl HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Result<Self, HttpError> {
        let buf_reader = BufReader::new(stream);
        let mut http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let body = http_request.pop().unwrap();
        let mut headers: Vec<Header> = Vec::new();
        while http_request.len() > 1 {
            headers.push(Header::from_str(&http_request.pop().unwrap()).unwrap())
        }
        headers.reverse();
        let request_line =
            RequestLine::from_str(&http_request.pop().unwrap()).unwrap();

        Ok(Self { request_line, headers, body })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct RequestLine {
    http_method: String,
    request_target: String,
    http_version: String,
}

impl RequestLine {
    fn from_str(request_line: &str) -> Result<Self, HttpError> {
        let mut split = request_line.split_whitespace();

        let http_method: String = split.next().unwrap().to_string();
        let request_target: String = split.next().unwrap().to_string();
        let http_version: String = split.next().unwrap().to_string();

        Ok(Self { http_method, request_target, http_version })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Header {
    info: String,
    content: String,
}

impl Header {
    fn from_str(header: &str) -> Result<Self, HttpError> {
        let mut split = header.split(": ");

        let info = split.next().unwrap().to_string();
        let content = split.next().unwrap().to_string();

        Ok(Self { info, content })
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}\r\n",
               self.info,
               self.content,
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct HttpResponse {
    status_line: StatusLine,
    header: Header,
    response_body: String,
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\r\n{}\r\n{}",
            self.status_line,
            self.header,
            self.response_body
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct StatusLine {
    http_version: String,
    status_code: StatusCode,
}

impl StatusLine {
    const HTTP_1_1: &'static str = "HTTP/1.1";
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}",
            self.http_version,
            self.status_code as usize,
            self.status_code.reason_phrase()
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum StatusCode {
    OK = 200,
    NotFound = 404,
}

impl StatusCode {
    fn reason_phrase(&self) -> String {
        match self {
            StatusCode::OK => "OK",
            StatusCode::NotFound => "Not Found",
        }.to_string()
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let http_request = HttpRequest::from_stream(&mut stream).unwrap();

    let response: &str =
        match http_request.request_line.request_target.as_str() {
        "/" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "HTTP/1.1 404 Not Found\r\n\r\n",
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn generate_response(status_code: StatusCode) -> String {
    let status = generate_status_line(status_code);
    //let header = generate_header();
    //let content = generate_body();

    //format!("{status}{header}{content}")
    todo!()
}

fn generate_status_line(status_code: StatusCode) -> StatusLine {
    format!("HTTP/1.1 200 OK\r\n");

    todo!()
}

fn generate_header() -> Header {
    format!("\r\n");
    todo!()
}

fn generate_body() -> String {
    format!("")
}
