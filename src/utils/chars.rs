use std::fmt::Write;

/// Replaces `?` placeholders in an SQL query with `$1`, `$2`, etc.
pub fn replace_placeholders(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut count = 1;
    let mut chars = sql.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '?' {
            let _ = result.write_str("$");
            let _ = result.write_str(&count.to_string());
            count += 1;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_placeholders() {
        let sql = "SELECT * FROM users WHERE id = ? AND name = ?";
        let new_sql = replace_placeholders(sql);
        assert_eq!(new_sql, "SELECT * FROM users WHERE id = $1 AND name = $2");
    }
    
}
