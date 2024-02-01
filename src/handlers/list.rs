use case_insensitive_hashmap::CaseInsensitiveHashMap;

use crate::utils_lib::index_of;

pub static BASE: &str = "/api/directory";
pub static METHOD: &str = "GET";
pub fn handle_list(
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
    let mut final_data = [].to_vec();
    if !literal_path.is_empty() {
        if literal_path.starts_with('.') {
            // idk if there is any side effect of this...
            literal_path = literal_path[1..].to_string();
        }
        let joined_path_buf =
            &std::path::PathBuf::from(mount_point).join(format!(".{}", literal_path));
        let meta = std::fs::metadata(joined_path_buf);
        if meta.is_ok() {
            if !meta.unwrap().is_dir() {
                error_reason = crate::handlers::errors::IS_FILE;
            } else if !super::is_path_within_mount_point(joined_path_buf, mount_point) {
                error_reason = crate::handlers::errors::OUT_OF_FS;
            } else {
                final_data.extend(
                    std::fs::read_dir(joined_path_buf)
                        .unwrap()
                        .filter_map(|entry| {
                            entry.ok().and_then(|e| {
                                let path = e.path();
                                path.file_name().and_then(|n| n.to_str().map(String::from))
                            })
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_bytes()
                        .to_vec(),
                );
            }
        } else {
            error_reason = crate::handlers::errors::GENERAL_FAILURE;
        }
    } else {
        error_reason = crate::handlers::errors::NO_INPUT;
    }
    crate::http_lib::ResponseDataToSet {
        base: crate::http_lib::BasicHTTPDataToSet {
            headers: super::all_origins(), // very unsafe, TODO
            data: final_data,
        },
        code: if error_reason == crate::handlers::errors::OK {
            200
        } else {
            400
        },
        msg: error_reason.to_string(),
    }
}
