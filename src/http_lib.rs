use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
extern crate chrono;
use chrono::{DateTime, Utc};

fn format_current_date() -> String {
    let current_utc_datetime: DateTime<Utc> = Utc::now();
    let formatted_date = current_utc_datetime
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();
    formatted_date
}

pub const COMMON_HTTP_METHODS: [&str; 5] = ["GET", "POST", "PUT", "DELETE", "PATCH"];

fn construct_http_response(additional_data: ResponseDataToSet) -> Vec<u8> {
    // fn construct_http_response(additional_data: RequestDataToSet) -> String {
    let http_header = format!("HTTP/1.1 {} {}", additional_data.code, additional_data.msg);
    let date = format_current_date();
    let mut headers: HashMap<String, String> = HashMap::new();
    if !additional_data.base.headers.is_empty() {
        headers.extend(additional_data.base.headers);
    }
    headers.insert("Date".to_string(), date);
    headers.insert(
        "Content-Length".to_string(),
        additional_data.base.data.len().to_string(),
    );
    // let headersRaw = String::new();
    let mut headers_raw: Vec<String> = Vec::new();
    for header in headers {
        headers_raw.push(format!("{}: {}", header.0, header.1));
    }
    let mut _final = format!(
        "{}\r\n{}\r\n\n",
        http_header,
        headers_raw.join("\r\n"),
        // String::from_utf8_lossy(&additional_data.data),
    )
    .as_bytes()
    .to_vec();
    // return _final;
    _final.extend(additional_data.base.data);
    _final
}

pub struct BasicHTTPDataToSet {
    pub headers: HashMap<String, String>,
    pub data: Vec<u8>,
}

pub struct ResponseDataToSet {
    pub base: BasicHTTPDataToSet,
    pub code: i32,
    pub msg: String,
}

pub struct RequestDataToSet {
    pub base: BasicHTTPDataToSet,
    pub method_and_path: String,
}
#[allow(clippy::unused_io_amount)]
fn handle_client(mut stream: TcpStream, handle_request: impl Fn(Vec<u8>) -> ResponseDataToSet) {
    let mut buffer = [0u8; 4096];
    stream.read(&mut buffer).unwrap();
    // stream.read_exact(&mut buffer).unwrap();
    let data = handle_request(buffer.to_vec());
    let response = construct_http_response(data);
    stream.write_all(response.as_ref()).unwrap();
}

fn extract_line(vec_bytes: &[u8], line_number: usize) -> Option<&[u8]> {
    let mut lines = vec![];
    let mut start = 0;

    for (i, &byte) in vec_bytes.iter().enumerate() {
        if byte == b'\n' {
            lines.push(&vec_bytes[start..i]);
            start = i + 1;
        }
    }

    if line_number < lines.len() {
        return Some(lines[line_number]);
    }
    None
}

fn index_of(haystack: &str, needle: &str) -> Option<usize> {
    let haystack_len = haystack.len();
    let needle_len = needle.len();

    if needle_len == 0 {
        return Some(0);
    }

    (0..=(haystack_len - needle_len)).find(|&i| &haystack[i..(i + needle_len)] == needle)
}

fn find_double_crlf(input: &[u8]) -> Option<usize> {
    input
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .find(|&i| input[i..].starts_with(b"\r\n\r\n"))
}

pub fn extract_path_and_method(method_and_path: &str, path: &mut String, method: &mut String) {
    for http_method in COMMON_HTTP_METHODS.iter() {
        if method_and_path.starts_with(http_method) {
            *method = method_and_path[..http_method.len()].trim_end().to_owned();
            *path = method_and_path[http_method.len()..].trim_start().to_owned();
            break;
        }
    }
}

pub fn start_server<F>(port: i32, req_handler: F)
where
    F: Fn(RequestDataToSet) -> ResponseDataToSet + Send + 'static,
{
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    let req_handler = std::sync::Arc::new(std::sync::Mutex::new(req_handler));

    println!("Server listening on http://127.0.0.1:{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let req_handler = std::sync::Arc::clone(&req_handler);

                thread::spawn(move || {
                    let req = |req_data: Vec<u8>| -> ResponseDataToSet {
                        // let data = String::from_utf8_lossy(&req_data);
                        // println!("{}", data);
                        // let first_line = extract_line(&req_data, 0).unwrap();
                        // let first_line_as_string = String::from_utf8_lossy(first_line);
                        let data_very_raw = req_data.split_at(find_double_crlf(&req_data).unwrap());
                        let headers_very_raw = data_very_raw.0;
                        let first_line = extract_line(headers_very_raw, 0).unwrap();
                        let first_line_as_string = String::from_utf8_lossy(first_line);
                        let binding = String::from_utf8_lossy(headers_very_raw);
                        let headers_raw = binding.split("\r\n");
                        let mut headers: HashMap<String, String> = HashMap::new();
                        for header in headers_raw {
                            let split_point = index_of(header, ":");
                            if split_point.is_none() {
                                continue;
                            }
                            headers.insert(
                                header[..split_point.unwrap()].to_string(),
                                header[split_point.unwrap() + 1..].trim_start().to_string(),
                            );
                        }
                        let incoming = RequestDataToSet {
                            method_and_path: first_line_as_string
                                [..index_of(&first_line_as_string, " HTTP").unwrap()]
                                .to_string(),
                            base: BasicHTTPDataToSet {
                                headers,
                                data: data_very_raw.1.to_vec(),
                            },
                        };
                        let guard = req_handler.lock().unwrap();
                        guard(incoming)
                    };

                    handle_client(stream, req);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
