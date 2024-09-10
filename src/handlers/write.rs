// use core::slice::SlicePattern;
use std::io::{Read, Write};

use bytes::BufMut;
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use may_minihttp::{Request, Response};
use percent_encoding::percent_decode_str;

use crate::utils_lib::index_of;

pub static BASE: &str = "/api/file";
pub static METHOD: &str = "POST";
pub fn handle_write(
    incoming_req: Request,
    mount_point: &str,
    response: &mut Response,
) -> std::io::Result<()> {
    let mut method = incoming_req.method();
    let mut path = incoming_req.path();
    response.header("Access-Control-Allow-Origin: *");

    let search_pos_opt = index_of(&path, "?");
    if search_pos_opt.is_none() {
        response.status_code(500, "Server error");
        return Ok(());
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
        literal_path = percent_decode_str(value).decode_utf8().unwrap().to_string();
    }
    let mut error_reason = crate::handlers::errors::OK;
    // let mut response_body = Vec::new();
    if !literal_path.is_empty() {
        if literal_path.starts_with('.') {
            // idk if there is any side effect of this...
            literal_path = literal_path[1..].to_string();
        }
        let joined_path_buf =
            &std::path::PathBuf::from(mount_point).join(format!("./{}", literal_path));
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
                let mut req_body = incoming_req.body();
                let mut file = res.unwrap();
                // let mut buffer = [0u8; 1024];
                let mut buffer = [0u8; 4096];
                loop {
                    let read_result = req_body.read(&mut buffer);
                    if read_result.is_err() {
                        error_reason = crate::handlers::errors::WRITE_ERROR;
                        break;
                    }
                    let bytes_read = read_result.ok().unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    let res2 = file.write_all(&buffer[..bytes_read]);
                    if res2.is_err() {
                        error_reason = crate::handlers::errors::WRITE_ERROR;
                    }
                }
                let truncate_result = file.set_len(req_body.body_limit() as u64);
                if truncate_result.is_err() {
                    error_reason = crate::handlers::errors::TRUNCATE_ERROR;
                }
                // let mut buf = Vec::new();
                // let read_result = req_body.read_to_end(&mut buf);
                // if read_result.is_err() {
                //     error_reason = crate::handlers::errors::WRITE_ERROR;
                // }
                // file.write_all(buf.as_slice());
                file.flush()?;
                // let res2 = res.unwrap().write_all();
                // response.extend(format!("{}", buffer_copy.len()).as_bytes());
                response.body(error_reason);
                return Ok(());
            }
        }
    } else {
        error_reason = crate::handlers::errors::NO_INPUT;
    }
    response.status_code(400, error_reason);
    Ok(())
}
