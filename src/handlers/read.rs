use bytes::BufMut;
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use may_minihttp::{Request, Response};
use percent_encoding::percent_decode_str;
use std::io::{Read, Write};

use crate::utils_lib::index_of;

pub static BASE: &str = "/api/file";
pub static METHOD: &str = "GET";
pub fn handle_read(
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
        // return crate::http_lib::ResponseDataToSet {
        //     base: crate::http_lib::BasicHTTPDataToSet {
        //         headers: CaseInsensitiveHashMap::new(),
        //         data: "wrong".as_bytes().to_vec(),
        //     },
        //     code: 500,
        //     msg: "Server error".to_string(),
        // };
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
    // let mut final_data = [].to_vec();
    if !literal_path.is_empty() {
        if literal_path.starts_with('.') {
            // idk if there is any side effect of this...
            literal_path = literal_path[1..].to_string();
        }
        let joined_path_buf =
            &std::path::PathBuf::from(mount_point).join(format!(".{}", literal_path));
        let meta = std::fs::metadata(joined_path_buf);
        if meta.is_ok() {
            if !meta.unwrap().is_file() {
                error_reason = crate::handlers::errors::IS_DIR;
            } else if !super::is_path_within_mount_point(joined_path_buf, mount_point) {
                error_reason = crate::handlers::errors::OUT_OF_FS;
            } else {
                // final_data.extend(std::fs::read(joined_path_buf).unwrap().to_vec());
                // let w = response.body_mut();
                let mut file = std::fs::File::open(joined_path_buf)?;
                // let _n = file.read_to_end(w.to_vec().as_mut())?;
                let mut w = response.body_mut().writer();
                let mut chunk = [0; 1024];
                loop {
                    match file.read(&mut chunk)? {
                        0 => break, // End of file
                        n => {
                            w.write_all(&chunk[..n])?;
                        }
                    }
                }
                return Ok(());
            }
        } else {
            error_reason = crate::handlers::errors::GENERAL_FAILURE;
        }
    } else {
        error_reason = crate::handlers::errors::NO_INPUT;
    }
    response.status_code(400, error_reason);
    Ok(())
    // crate::http_lib::ResponseDataToSet {
    //     base: crate::http_lib::BasicHTTPDataToSet {
    //         headers: super::all_origins(), // very unsafe, TODO
    //         data: final_data,
    //     },
    //     code: if error_reason == crate::handlers::errors::OK {
    //         200
    //     } else {
    //         400
    //     },
    //     msg: error_reason.to_string(),
    // }
}
