use std::io::Write;

use bytes::BufMut;
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use may_minihttp::{Request, Response};
use percent_encoding::percent_decode_str;

use crate::utils_lib::{self, index_of};

pub static BASE: &str = "/api/stat";
pub static METHOD: &str = "GET";
pub fn handle_stat(
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
    // let mut final_data = Vec::new();
    if !literal_path.is_empty() {
        if literal_path.starts_with('.') {
            // idk if there is any side effect of this...
            literal_path = literal_path[1..].to_string();
        }
        let joined_path_buf =
            &std::path::PathBuf::from(mount_point).join(format!(".{}", literal_path));
        let meta = std::fs::metadata(joined_path_buf);
        if meta.is_ok() {
            let true_meta = meta.as_ref().unwrap();
            /*if !true_meta.is_file() {
                error_reason = crate::handlers::errors::IS_DIR;
            } else*/
            if !super::is_path_within_mount_point(joined_path_buf, mount_point) {
                error_reason = crate::handlers::errors::OUT_OF_FS;
            } else {
                // final_data.extend(std::fs::read(joined_path_buf).unwrap().to_vec());
                let mut out_raw = std::collections::HashMap::new();
                out_raw.insert(
                    "itemType",
                    (crate::utils_lib::file_type_convert(true_meta.file_type()).unwrap() as u32)
                        .to_string(),
                );
                out_raw.insert("size", true_meta.len().to_string());
                out_raw.insert("mode", crate::utils_lib::get_mode(true_meta).to_string());
                let true_creation_time: u64;
                #[cfg(unix)]
                {
                    true_creation_time = std::os::unix::fs::MetadataExt::ctime(true_meta) as u64;
                };
                #[cfg(not(unix))]
                {
                    true_creation_time = true_meta
                        .created()
                        .unwrap()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards!")
                        .as_secs();
                };
                let true_modification_time: u64;
                #[cfg(unix)]
                {
                    true_modification_time =
                        std::os::unix::fs::MetadataExt::mtime(true_meta) as u64;
                };
                #[cfg(not(unix))]
                {
                    true_modification_time = true_meta
                        .modified()
                        .unwrap()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards!")
                        .as_secs();
                };
                let true_access_time: u64;
                #[cfg(unix)]
                {
                    true_access_time = std::os::unix::fs::MetadataExt::atime(true_meta) as u64;
                };
                #[cfg(not(unix))]
                {
                    true_access_time = true_meta
                        .accessed()
                        .unwrap()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards!")
                        .as_secs();
                };
                out_raw.insert("ctime", true_creation_time.to_string());
                out_raw.insert("mtime", true_modification_time.to_string());
                out_raw.insert("atime", true_access_time.to_string());
                let mut w = response.body_mut().writer();
                // final_data.extend(utils_lib::create_options(out_raw).as_bytes());
                w.write_all(utils_lib::create_options(out_raw).as_bytes())?;
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
}
