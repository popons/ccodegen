use std::path::Path;

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Get the file name from a path
pub fn get_file_name(path: &Path) -> Option<String> {
    path.file_name().and_then(|name| name.to_str()).map(String::from)
}

/// Join strings with a separator
pub fn join_strings<S: AsRef<str>>(strings: &[S], separator: &str) -> String {
    strings
        .iter()
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>()
        .join(separator)
}

/// Repeat a string n times
pub fn repeat_str(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Convert a string to a valid identifier
pub fn to_valid_identifier(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    
    // First character must be a letter or underscore
    if let Some(first) = chars.next() {
        if first.is_alphabetic() || first == '_' {
            result.push(first);
        } else {
            result.push('_');
        }
    }
    
    // Remaining characters can be alphanumeric or underscore
    for c in chars {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    
    result
}

/// Ensure a string ends with a newline
pub fn ensure_ends_with_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}
