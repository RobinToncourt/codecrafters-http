#![allow(dead_code)]
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

const CRLF: &str = "\r\n";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum HttpError {

}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct HttpRequest {
    request_line: RequestLine,
    headers: Vec<Header>,
    request_body: RequestBody,
}

impl HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Result<Self, HttpError> {
        let buf_reader = BufReader::new(stream);
        let mut http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let request_body = RequestBody(http_request.pop().unwrap());
        let mut headers: Vec<Header> = Vec::new();
        while http_request.len() > 1 {
            headers.push(Header::from_str(&http_request.pop().unwrap()).unwrap())
        }
        headers.reverse();
        let request_line =
            RequestLine::from_str(&http_request.pop().unwrap()).unwrap();

        Ok(Self { request_line, headers, request_body })
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
    name: String,
    value: String,
}

impl Header {
    fn from_str(header: &str) -> Result<Self, HttpError> {
        let mut split = header.split(": ");

        let name = split.next().unwrap().to_string();
        let value = split.next().unwrap().to_string();

        Ok(Self { name, value })
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}{CRLF}",
               self.name,
               self.value,
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct RequestBody(String);

impl fmt::Display for RequestBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct HttpResponse {
    status_line: StatusLine,
    headers: Vec<Header>,
    response_body: ResponseBody,
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{CRLF}{}{CRLF}{}",
            self.status_line,
            join_headers(&self.headers),
            self.response_body
        )
    }
}

fn join_headers(headers: &Vec<Header>) -> String {
    let mut joined_headers = String::new();

    headers.iter()
        .for_each(
            |header| joined_headers.push_str(&header.to_string())
        );

    joined_headers
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct StatusLine {
    http_version: String,
    status_code: StatusCode,
}

impl StatusLine {
    const HTTP_1_1: &'static str = "HTTP/1.1";
}

impl StatusLine {
    fn new(status_code: StatusCode) -> Self {
        Self {
            http_version: StatusLine::HTTP_1_1.to_string(),
            status_code,
        }
    }
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct ResponseBody(String);

impl fmt::Display for ResponseBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

    let response: String =
        if http_request.request_line.request_target.eq("/") {
        format!("HTTP/1.1 200 OK{CRLF}{CRLF}")
    }
    else if http_request.request_line.request_target.starts_with("/echo/") {
        let (mut headers, response_body): (Vec<Header>, ResponseBody) =
            echo_page(&http_request);

        let content_type = Header {
            name: "Content-Type".to_string(),
            value: "text/plain".to_string(),
        };
        headers.insert(0, content_type);

        let status_line = StatusLine::new(StatusCode::OK);

        let http_response = HttpResponse {
            status_line,
            headers,
            response_body,
        };

        format!("{http_response}")
    }
    else {
        format!("HTTP/1.1 404 Not Found{CRLF}{CRLF}")
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn echo_page(http_request: &HttpRequest) -> (Vec<Header>, ResponseBody) {
    let split: Vec<&str> =
        http_request.request_line.request_target.split("/").collect();

    let response_body = ResponseBody(split[split.len()-1].to_string());

    let content_length = Header {
        name: "Content-Length".to_string(),
        value: response_body.0.len().to_string(),
    };
    let headers: Vec<Header> = vec![content_length];

    (headers, response_body)
}






