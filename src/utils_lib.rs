pub fn index_of(haystack: &str, needle: &str) -> Option<usize> {
    let haystack_len = haystack.len();
    let needle_len = needle.len();

    if needle_len == 0 {
        return Some(0);
    }

    (0..=(haystack_len - needle_len)).find(|&i| &haystack[i..(i + needle_len)] == needle)
}
pub fn truncate_buffer(buffer: &mut Vec<u8>, size: usize) {
    if buffer.len() > size {
        buffer.truncate(size);
    }
}
pub fn create_options(input: std::collections::HashMap<&str, String>) -> std::string::String {
    let mut full = String::new();
    for (key, val) in input {
        full.push_str(&format!("{}=\"{}\"\n", key, val));
    }
    full
}
pub enum FileType {
    File = 0x8000,
    Directory = 0x4000,
    Symlink = 0xA000,
}

pub fn file_type_convert(file_type: std::fs::FileType) -> Option<FileType> {
    if file_type.is_file() {
        Some(FileType::File)
    } else if file_type.is_dir() {
        Some(FileType::Directory)
    } else if file_type.is_symlink() {
        Some(FileType::Symlink)
    } else {
        None
    }
}

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(not(unix))]
fn fake_mode() -> u32 {
    0o755
}

#[cfg(unix)]
pub fn get_mode(meta: &std::fs::Metadata) -> u32 {
    meta.mode()
}

#[cfg(not(unix))]
pub fn get_mode(_: &fs::Metadata) -> u32 {
    fake_mode()
}
