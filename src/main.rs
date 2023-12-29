use std::collections::HashMap;
mod http_lib;

fn main() {
    let handler = |incoming_req: http_lib::RequestDataToSet| -> http_lib::ResponseDataToSet {
        println!("got request");
        let mut method: &str = "GET";
        let mut path: &str = "/";
        for http_method in http_lib::COMMON_HTTP_METHODS {
            if incoming_req.method_and_path.starts_with(http_method) {
                method = &incoming_req.method_and_path[..http_method.chars().count()].trim_end();
                path = &incoming_req.method_and_path[http_method.chars().count()..].trim_start();
                break;
            }
        }
        println!("method: {}\npath: {}", method, path);
        for (key, value) in &incoming_req.base.headers {
            println!("header: {}: {}", key, value);
        }
        let send_error = |code: i32, msg: &str| http_lib::ResponseDataToSet {
            base: http_lib::BasicHTTPDataToSet {
                headers: HashMap::new(),
                data: [].to_vec(),
            },
            code,
            msg: msg.to_string(),
        };
        match method {
            "GET" => {
                if let Ok(meta) = std::fs::metadata(format!(".{}", path)) {
                    if meta.is_dir() {
                        return send_error(503, "Server error");
                    }
                    return http_lib::ResponseDataToSet {
                        base: http_lib::BasicHTTPDataToSet {
                            headers: HashMap::new(),
                            // data: "sigma".as_bytes().to_vec(),
                            data: std::fs::read(format!(".{}", path)).unwrap(),
                        },
                        code: 200,
                        msg: "OK".to_string(),
                    };
                }
                send_error(404, "Not Found")
            }
            _ => send_error(503, "Server error"),
        }
    };
    http_lib::start_server(8080, handler);
}
