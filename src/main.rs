use case_insensitive_hashmap::CaseInsensitiveHashMap;
mod http_lib;

mod handlers;
mod utils_lib;

fn main() {
    let handler = |incoming_req: http_lib::RequestDataToSet| -> http_lib::ResponseDataToSet {
        println!("got request");
        let mut method = "GET".to_string();
        let mut path = "/".to_string();
        // for http_method in http_lib::COMMON_HTTP_METHODS {
        //     if incoming_req.method_and_path.starts_with(http_method) {
        //         method = &incoming_req.method_and_path[..http_method.chars().count()].trim_end();
        //         path = &incoming_req.method_and_path[http_method.chars().count()..].trim_start();
        //         break;
        //     }
        // }
        http_lib::extract_path_and_method(
            incoming_req.method_and_path.as_str(),
            &mut path,
            &mut method,
        );
        // println!("method: {}\npath: {}", method, path);
        // for (key, value) in &incoming_req.base.headers {
        //     println!("header: {}: {}", key, value);
        // }
        let send_error = |code: i32, msg: &str| http_lib::ResponseDataToSet {
            base: http_lib::BasicHTTPDataToSet {
                headers: CaseInsensitiveHashMap::new(),
                data: [].to_vec(),
            },
            code,
            msg: msg.to_string(),
        };
        // let method_str = method.as_str();
        if path.starts_with(handlers::read::BASE) && method == handlers::read::METHOD {
            return handlers::read::handle_read(
                incoming_req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
            );
        }
        if path.starts_with(handlers::list::BASE) && method == handlers::list::METHOD {
            return handlers::list::handle_list(
                incoming_req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
            );
        }
        if path.starts_with(handlers::write::BASE) && method == handlers::write::METHOD {
            return handlers::write::handle_write(
                incoming_req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
            );
        }
        if path.starts_with(handlers::stat::BASE) && method == handlers::stat::METHOD {
            return handlers::stat::handle_stat(
                incoming_req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
            );
        }
        send_error(500, "Server error")
    };
    // http_lib::start_server(8080, handler);
    http_lib::start_server(2137, handler);
}
