use std::io::Write;

use case_insensitive_hashmap::CaseInsensitiveHashMap;

use crate::utils_lib::index_of;

pub static BASE: &str = "/api/file";
pub static METHOD: &str = "POST";
pub fn handle_write(
    incoming_req: crate::http_lib::RequestDataToSet,
    mount_point: &str,
) -> crate::http_lib::ResponseDataToSet {
    let mut method = "GET".to_string();
    let mut path = "/".to_string();
    crate::http_lib::extract_path_and_method(
        incoming_req.method_and_path.as_str(),
        &mut path,
        &mut method,
    );
    let search_pos_opt = index_of(&path, "?");
    if search_pos_opt.is_none() {
        return crate::http_lib::ResponseDataToSet {
            base: crate::http_lib::BasicHTTPDataToSet {
                headers: CaseInsensitiveHashMap::new(),
                data: "wrong".as_bytes().to_vec(),
            },
            code: 500,
            msg: "Server error".to_string(),
        };
    }
    let search_pos = search_pos_opt.unwrap();
    let search = &path[search_pos..];
    // let mut out: Vec<String> = Vec::new();
    let mut literal_path = String::new();
    for (key, value) in
        // crate::http_lib::parse_search_options(search.strip_suffix('?').unwrap_or(search))
        crate::http_lib::parse_search_options(&search[1..])
    {
        // println!("{}: {}", key, value);
        // out.push(format!("{}: {}", key, value));
        if key != "path" {
            continue;
        }
        literal_path = value.to_string();
    }
    let mut error_reason = crate::handlers::errors::OK;
    let mut response = Vec::new();
    if !literal_path.is_empty() {
        if literal_path.starts_with('.') {
            // idk if there is any side effect of this...
            literal_path = literal_path[1..].to_string();
        }
        let joined_path_buf =
            &std::path::PathBuf::from(mount_point).join(format!(".{}", literal_path));
        dbg!(joined_path_buf);
        let meta = std::fs::metadata(joined_path_buf);
        // if meta.is_err() {
        //     let res = std::fs::File::create(joined_path_buf);
        //     if res.is_err() {
        //         error_reason = "File didn't exist, creation caused errors";
        //     }
        //     meta = std::fs::metadata(joined_path_buf);
        // }
        // if meta.is_ok()
        //     && meta.unwrap().is_file()
        //     && super::is_path_within_mount_point(joined_path_buf, mount_point)
        // {
        //     let res = std::fs::write(joined_path_buf, incoming_req.base.data);
        //     if res.is_err() {
        //         error_reason = "Write error";
        //     }
        // } else if error_reason == "OK" {
        //     error_reason = "Out of scope or doesn't exist or is a dir";
        // }
        if !super::is_path_within_mount_point(joined_path_buf, mount_point) {
            error_reason = crate::handlers::errors::OUT_OF_FS;
        } else if meta.is_ok() && meta.unwrap().is_dir() {
            error_reason = crate::handlers::errors::IS_DIR;
        } else {
            let mut res = std::fs::File::options()
                .read(true)
                .write(true)
                .open(joined_path_buf);
            let mut everything_fine = false;
            if res.is_err() {
                if let Err(ref err) = res {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        // println!("error, good");
                        res = std::fs::File::create(joined_path_buf);
                        if res.is_err() {
                            error_reason = crate::handlers::errors::CREATION_ERROR;
                        } else {
                            everything_fine = true;
                        }
                    }
                }
                if !everything_fine {
                    error_reason = crate::handlers::errors::OPEN_ERROR;
                }
            } else {
                // println!("how did we get here");
                everything_fine = true;
            }
            if everything_fine {
                let mut buffer_copy = incoming_req.base.data.clone();
                crate::utils_lib::truncate_buffer(
                    &mut buffer_copy,
                    incoming_req
                        .base
                        .headers
                        .get("Content-Length")
                        .unwrap()
                        .parse::<usize>()
                        .unwrap(),
                );
                dbg!(&buffer_copy);
                let res2 = res.unwrap().write_all(buffer_copy.as_slice());
                if res2.is_err() {
                    error_reason = crate::handlers::errors::WRITE_ERROR;
                }
                response.extend(format!("{}", buffer_copy.len()).as_bytes());
            }
        }
    } else {
        error_reason = crate::handlers::errors::NO_INPUT;
    }
    crate::http_lib::ResponseDataToSet {
        base: crate::http_lib::BasicHTTPDataToSet {
            headers: super::all_origins(), // very unsafe, TODO
            data: response,
        },
        code: if error_reason == crate::handlers::errors::OK {
            200
        } else {
            400
        },
        msg: error_reason.to_string(),
    }
}
