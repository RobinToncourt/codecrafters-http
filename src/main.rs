#![allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{prelude::*, Error, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use flate2::write::GzEncoder;
use flate2::Compression;

use either::*;

const CRLF: &str = "\r\n";
const SUPPORTED_ENCODING: [&str; 1] = ["gzip"];

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum HttpError {
    UnknowHttpMethod,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct HttpRequest {
    request_line: RequestLine,
    headers: HashMap<String, String>,
    request_body: Option<String>,
}

impl HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Result<Self, HttpError> {
        let http_request: Vec<String> = read_stream_to_string(stream).unwrap();

        let request_line = RequestLine::from_str(&http_request[0]).unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut index: usize = 1;
        loop {
            let Some(tmp) = &http_request.get(index) else {
                break;
            };
            let Some((key, value)) = parse_header(tmp) else {
                break;
            };
            headers.insert(key, value);
            index += 1;
        }

        index += 1;

        let request_body = http_request.get(index).cloned();

        Ok(Self {
            request_line,
            headers,
            request_body,
        })
    }
}

fn parse_header(header: &str) -> Option<(String, String)> {
    let mut split = header.split(": ");

    let name = split.next()?;
    let value = split.next()?;

    Some((name.to_string(), value.to_string()))
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
    Ok(content
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
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

        let http_method: HttpMethod = HttpMethod::from_str(split.next().unwrap())?;
        let request_target: String = split.next().unwrap().to_string();
        let http_version: String = split.next().unwrap().to_string();

        Ok(Self {
            http_method,
            request_target,
            http_version,
        })
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

#[derive(PartialEq, Eq, Debug)]
struct HttpResponse {
    status_line: StatusLine,
    headers: HashMap<String, String>,
    response_body: Either<String, Vec<u8>>,
}

impl HttpResponse {
    //const NOT_FOUND: HttpResponse

    fn write_all(&self, stream: &mut TcpStream) {
        stream
            .write_all(
                &format!(
                    "{}{CRLF}{}{CRLF}",
                    self.status_line,
                    join_headers(&self.headers),
                )
                .into_bytes(),
            )
            .unwrap();

        match &self.response_body {
            Either::Left(s) => stream.write_all(s.as_bytes()).unwrap(),
            Either::Right(v) => stream.write_all(v).unwrap(),
        }
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rbs: String = match &self.response_body {
            Either::Left(s) => s.clone(),
            Either::Right(v) => format!("{v:?}"),
        };

        write!(
            f,
            "{}{CRLF}{}{CRLF}{}",
            self.status_line,
            join_headers(&self.headers),
            rbs
        )
    }
}

fn join_headers(headers: &HashMap<String, String>) -> String {
    let mut joined_headers = String::new();

    headers
        .iter()
        .for_each(|(name, value)| joined_headers.push_str(&format!("{name}: {value}{CRLF}")));

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
        write!(
            f,
            "{} {} {}",
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
        }
        .to_string()
    }
}

fn byte_array_to_hex_string(arr: Vec<u8>) -> String {
    match String::from_utf8(arr) {
        Ok(s) => s,
        Err(e) => panic!("Invalid UTF-8 sequence: {e}"),
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

    let (status_line, mut headers, response_body): (
        StatusLine,
        HashMap<String, String>,
        Option<String>,
    ) = if request_target.eq("/") {
        let status_line = StatusLine::new(StatusCode::OK);
        let headers: HashMap<String, String> = HashMap::new();
        let response_body = None;

        (status_line, headers, response_body)
    } else if request_target.starts_with("/echo/") {
        let (headers, response_body): (HashMap<String, String>, Option<String>) =
            echo_page(&http_request);

        let status_line = StatusLine::new(StatusCode::OK);

        (status_line, headers, response_body)
    } else if request_target.eq("/user-agent") {
        let (headers, response_body): (HashMap<String, String>, Option<String>) =
            user_agent_page(&http_request);

        let status_line = StatusLine::new(StatusCode::OK);

        (status_line, headers, response_body)
    } else if request_target.starts_with("/files/")
        && http_request.request_line.http_method.eq(&HttpMethod::Get)
    {
        match get_file_page(&http_request, args) {
            Ok((headers, response_body)) => {
                let status_line = StatusLine::new(StatusCode::OK);

                (status_line, headers, response_body)
            }
            Err(_) => {
                let status_line = StatusLine::new(StatusCode::NotFound);
                let headers: HashMap<String, String> = HashMap::new();
                let response_body = None;

                (status_line, headers, response_body)
            }
        }
    } else if request_target.starts_with("/files/")
        && http_request.request_line.http_method.eq(&HttpMethod::Post)
    {
        match post_file_page(&http_request, args) {
            Ok((headers, response_body)) => {
                let status_line = StatusLine::new(StatusCode::Created);

                (status_line, headers, response_body)
            }
            Err(_) => {
                let status_line = StatusLine::new(StatusCode::NotFound);
                let headers: HashMap<String, String> = HashMap::new();
                let response_body = None;

                (status_line, headers, response_body)
            }
        }
    } else {
        let status_line = StatusLine::new(StatusCode::NotFound);
        let headers: HashMap<String, String> = HashMap::new();
        let response_body = None;

        (status_line, headers, response_body)
    };

    let response_body: Either<String, Vec<u8>> = encode_body(
        &http_request,
        &mut headers,
        response_body.unwrap_or(String::new()),
    );

    match &response_body {
        Either::Left(s) => headers.insert("Content-Length".to_string(), s.len().to_string()),
        Either::Right(v) => headers.insert("Content-Length".to_string(), v.len().to_string()),
    };

    let http_response = HttpResponse {
        status_line,
        headers,
        response_body,
    };

    http_response.write_all(&mut stream);
}

fn echo_page(http_request: &HttpRequest) -> (HashMap<String, String>, Option<String>) {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let response_body = Some(split[split.len() - 1].to_string());

    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());

    (headers, response_body)
}

fn user_agent_page(http_request: &HttpRequest) -> (HashMap<String, String>, Option<String>) {
    let user_agent_value: &str = http_request
        .headers
        .get("User-Agent")
        .expect("No User-Agent header");

    let response_body = Some(user_agent_value.to_string());

    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());

    (headers, response_body)
}

fn get_file_page(
    http_request: &HttpRequest,
    args: Vec<String>,
) -> std::io::Result<(HashMap<String, String>, Option<String>)> {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let filename = format!("{}{}", args[2], split[2]);

    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let response_body = Some(content);

    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert(
        "Content-Type".to_string(),
        "application/octet-stream".to_string(),
    );

    Ok((headers, response_body))
}

fn post_file_page(
    http_request: &HttpRequest,
    args: Vec<String>,
) -> std::io::Result<(HashMap<String, String>, Option<String>)> {
    let split: Vec<&str> = http_request
        .request_line
        .request_target
        .split("/")
        .collect();

    let filename = format!("{}{}", args[2], split[2]);

    let mut file = File::create(filename)?;
    file.write_all(http_request.request_body.clone().unwrap().as_bytes())?;

    let headers: HashMap<String, String> = HashMap::new();
    let response_body = None;

    Ok((headers, response_body))
}

fn encode_body(
    http_request: &HttpRequest,
    headers: &mut HashMap<String, String>,
    body: String,
) -> Either<String, Vec<u8>> {
    let encodings: &str = http_request
        .headers
        .get("Accept-Encoding")
        .map_or("", String::as_str);

    let client_encodings: Vec<&str> = encodings.split(", ").collect();

    let common_supported_encoding: Vec<&str> =
        common_str_elements(&SUPPORTED_ENCODING, &client_encodings);

    let encoding: &str = common_supported_encoding.first().unwrap_or(&"");

    let body: Either<String, Vec<u8>> = match encoding {
        "gzip" => {
            headers.insert("Content-Encoding".to_string(), "gzip".to_string());
            Right(gzip_encoding(body))
        }
        _ => Left(body),
    };

    body
}

fn common_str_elements<'a>(server: &[&'a str], client: &'a [&'a str]) -> Vec<&'a str> {
    let mut result: Vec<&str> = Vec::new();

    for s in server {
        if client.contains(s) {
            result.push(s);
        }
    }

    result
}

fn gzip_encoding(body: String) -> Vec<u8> {
    let mut gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gzip_encoder.write_all(body.as_bytes()).unwrap();
    gzip_encoder.finish().unwrap()
}
