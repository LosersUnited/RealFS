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
