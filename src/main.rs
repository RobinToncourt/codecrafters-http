#![allow(dead_code)]
use std::thread;
use std::env;
use std::fmt;
use std::fs::File;
use std::collections::HashMap;
use std::io::{
    Error,
    prelude::*,
    Write,
};
use std::net::{
    TcpStream,
    TcpListener,
};

const CRLF: &str = "\r\n";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum HttpError {
    UnknowHttpMethod,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct HttpRequest {
    request_line: RequestLine,
    headers: HashMap<String, String>,
    request_body: RequestBody,
}

impl HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Result<Self, HttpError> {
        // let buf_reader = BufReader::new(stream);
        // let http_request: Vec<String> = buf_reader
        //     .lines()
        //     .map(|result| result.unwrap())
        //     .take_while(|line| !line.is_empty())
        //     .collect();

        let http_request: Vec<String> = read_stream_to_string(stream).unwrap();

        println!("{http_request:?}");

        let request_line =
            RequestLine::from_str(&http_request[0]).unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut index: usize = 1;
        loop {
            let Some(tmp) = &http_request.get(index) else {
                break;
            };
            let Some((key, value)) = Header::parse(tmp) else {
                break;
            };
            headers.insert(key, value);
            index += 1;
        }

        index += 1;

        let request_body = RequestBody(
            if let Some(tmp) = &http_request.get(index) {
                tmp
            }
            else {
                ""
            }.to_string());

        println!("{request_line:?}\n{headers:?}\n{request_body:?}");

        Ok(Self { request_line, headers, request_body })
    }
}

fn read_stream_to_string(stream: &mut TcpStream) -> Result<Vec<String>, Error> {
    let mut data: Vec<u8> = Vec::new();

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let nb_bytes: usize = stream.read(&mut buffer)?;

        buffer.into_iter().take(nb_bytes).for_each(|b| data.push(b));

        if nb_bytes < 1024 {
            break;
        }
    }

    let content: String = String::from_utf8(data).unwrap();
    Ok(content.lines().map(|s| s.to_string()).collect::<Vec<String>>())
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct RequestLine {
    http_method: HttpMethod,
    request_target: String,
    http_version: String,
}

impl RequestLine {
    fn from_str(request_line: &str) -> Result<Self, HttpError> {
        let mut split = request_line.split_whitespace();

        let http_method: HttpMethod =
            HttpMethod::from_str(split.next().unwrap())?;
        let request_target: String = split.next().unwrap().to_string();
        let http_version: String = split.next().unwrap().to_string();

        Ok(Self { http_method, request_target, http_version })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    fn from_str(s: &str) -> Result<Self, HttpError> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(HttpError::UnknowHttpMethod),
        }
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

impl HttpResponse {
    //const NOT_FOUND: HttpResponse
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

fn join_headers(headers: &[Header]) -> String {
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
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
}

impl StatusCode {
    fn reason_phrase(&self) -> String {
        match self {
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::NotFound => "Not Found",
        }.to_string()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Header {
    name: String,
    value: String,
}

impl Header {
    fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    fn parse(header: &str) -> Option<(String, String)> {
        let mut split = header.split(": ");

        let name = split.next()?;
        let value = split.next()?;

        Some((name.to_string(), value.to_string()))
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
struct ResponseBody(String);

impl fmt::Display for ResponseBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();


    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let cloned_args = args.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream, cloned_args);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, args: Vec<String>) {
    let http_request = HttpRequest::from_stream(&mut stream).unwrap();

    let request_target: &str = &http_request.request_line.request_target;
    let response: String = if request_target.eq("/") {
        format!("HTTP/1.1 200 OK{CRLF}{CRLF}")
    }
    else if request_target.starts_with("/echo/") {
        let (headers, response_body): (Vec<Header>, ResponseBody) =
        echo_page(&http_request);

        let status_line = StatusLine::new(StatusCode::OK);

        let http_response = HttpResponse {
            status_line,
            headers,
            response_body,
        };

        format!("{http_response}")
    }
    else if request_target.eq("/user-agent") {
        let (headers, response_body): (Vec<Header>, ResponseBody) =
            user_agent_page(&http_request);

        let status_line = StatusLine::new(StatusCode::OK);

        let http_response = HttpResponse {
            status_line,
            headers,
            response_body,
        };

        format!("{http_response}")
    }
    else if request_target.starts_with("/files/")
        && http_request.request_line.http_method.eq(&HttpMethod::Get) {
        match get_file_page(&http_request, args) {
            Ok((headers, response_body)) => {
                let status_line = StatusLine::new(StatusCode::OK);

                let http_response = HttpResponse {
                    status_line,
                    headers,
                    response_body,
                };

                format!("{http_response}")
            },
            Err(_) => format!("HTTP/1.1 404 Not Found{CRLF}{CRLF}"),
        }
    }
    else if request_target.starts_with("/files/")
        && http_request.request_line.http_method.eq(&HttpMethod::Post) {
        match post_file_page(&http_request, args) {
            Ok((headers, response_body)) => {
                let status_line = StatusLine::new(StatusCode::Created);

                let http_response = HttpResponse {
                    status_line,
                    headers,
                    response_body,
                };

                format!("{http_response}")
            },
            Err(_) => format!("HTTP/1.1 404 Not Found{CRLF}{CRLF}"),
        }
    }
    else {
        format!("HTTP/1.1 404 Not Found{CRLF}{CRLF}")
    };

    println!("{response}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn echo_page(http_request: &HttpRequest) -> (Vec<Header>, ResponseBody) {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let response_body = ResponseBody(split[split.len()-1].to_string());

    let content_type = Header::new(
        "Content-Type".to_string(),
                                   "text/plain".to_string(),
    );
    let content_length = Header::new(
        "Content-Length".to_string(),
                                     response_body.0.len().to_string(),
    );
    let headers: Vec<Header> = vec![content_type, content_length];

    (headers, response_body)
}

fn user_agent_page(http_request: &HttpRequest) -> (Vec<Header>, ResponseBody) {
    let user_agent_value: &str =
    http_request.headers.get("User-Agent").expect("No User-Agent header");

    let response_body = ResponseBody(user_agent_value.to_string());

    let content_type = Header::new(
        "Content-Type".to_string(), "text/plain".to_string(),
    );
    let content_length = Header::new(
        "Content-Length".to_string(), response_body.0.len().to_string(),
    );
    let headers: Vec<Header> = vec![content_type, content_length];

    (headers, response_body)
}

fn get_file_page(
    http_request: &HttpRequest,
    args: Vec<String>,
) -> std::io::Result<(Vec<Header>, ResponseBody)> {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let filename = format!("{}{}", args[2], split[2]);

    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let response_body = ResponseBody(content);

    let content_type = Header::new(
        "Content-Type".to_string(), "application/octet-stream".to_string(),
    );
    let content_length = Header::new(
        "Content-Length".to_string(), response_body.0.len().to_string(),
    );

    let headers: Vec<Header> = vec![content_type, content_length];

    Ok((headers, response_body))
}

fn post_file_page(
    http_request: &HttpRequest,
    args: Vec<String>,
) -> std::io::Result<(Vec<Header>, ResponseBody)> {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let filename = format!("{}{}", args[2], split[2]);

    let mut file = File::create(filename)?;
    file.write_all(http_request.request_body.0.as_bytes())?;

    let headers: Vec<Header> = Vec::new();
    let response_body = ResponseBody(String::new());

    Ok((headers, response_body))
}
