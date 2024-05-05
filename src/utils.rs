// Trim double quotes and unescape string
pub fn unescape_json_string(escaped_json: &str) -> Result<String, serde_json::Error> {
    let trimmed = escaped_json.trim_matches('"');
    let unescaped = serde_json::from_str::<String>(&format!("\"{}\"", trimmed))?;
    Ok(unescaped)
}
