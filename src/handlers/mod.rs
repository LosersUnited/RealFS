pub mod list;
pub mod read;
pub mod write;

fn custom_canonicalize(path: &std::path::Path) -> std::path::PathBuf {
    let mut canonical_path = std::path::PathBuf::new();
    canonical_path.push("/"); // is this good?
    for component in path.components() {
        match component {
            std::path::Component::Normal(p) => canonical_path.push(p),
            std::path::Component::ParentDir => {
                let _ = canonical_path.pop();
            }
            _ => {}
        }
    }
    canonical_path
}
pub fn is_path_within_mount_point(path: &std::path::Path, mount_point: &str) -> bool {
    let canonical_mount_point = std::fs::canonicalize(mount_point).unwrap();
    let canonical_path = custom_canonicalize(path);
    dbg!(&canonical_path);
    canonical_path.starts_with(canonical_mount_point)
}
