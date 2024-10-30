pub fn strip_adb_prefix(response: String) -> String {
    if response.starts_with('$') || response.starts_with('+') {
        response[1..].to_string()
    } else {
        response
    }
}