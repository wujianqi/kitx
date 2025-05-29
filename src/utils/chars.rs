use std::{any::type_name, fmt::Write};

/// Replaces `?` placeholders in SQL query with PostgreSQL-style numbered parameters ($1, $2, etc.)
/// 
/// # Arguments
/// * `sql` - Original SQL string containing `?` placeholders
/// 
/// # Returns
/// New SQL string with numbered parameters
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

/// Returns the name of the given type
/// 
 /// # Arguments
 /// * `t` - Type to get the name of
 /// 
 /// # Returns
 /// Name of the given type
pub fn get_type_name<T>() -> &'static str {
    //let name = type_name_of_val(t);
    let name = type_name::<T>();
    name.rsplit("::").next().unwrap_or("")
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

    #[test]
    fn test_get_type_name() {
        assert_eq!(get_type_name::<String>(), "String");
    }
    
}
