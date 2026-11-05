pub fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(value: &[u16]) -> String {
    let length = value
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..length])
}
