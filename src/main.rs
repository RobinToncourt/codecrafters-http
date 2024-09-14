#[allow(unused_imports)]
use std::io::Write;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response: String = generate_response();
                stream.write(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn generate_response() -> String {
    let status = generate_status_line();
    let header = generate_header();
    let content = generate_body();

    format!("{status}{header}{content}")
}

fn generate_status_line() -> String {
    format!("HTTP/1.1 200 OK\r\n")
}

fn generate_header() -> String {
    format!("\r\n")
}

fn generate_body() -> String {
    format!("")
}
