use std::{any::Any, fmt::Write};

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use std::cmp::max;

/// Helper function to recursively unwrap any number of Option layers
/// and return the inner value if it exists.
/// This function is useful when dealing with nested Option types.
/// It can handle `Option<Option<T>>`, `Option<T>`, and T types.
pub fn unwrap_option<'a, T: 'static>(value: &'a dyn Any) -> Option<&'a T> {
    if let Some(opt_opt) = value.downcast_ref::<Option<Option<T>>>() {
        return opt_opt.as_ref().and_then(|opt| opt.as_ref());
    }
    if let Some(opt) = value.downcast_ref::<Option<T>>() {
        return opt.as_ref();
    }

    value.downcast_ref::<T>()
}

/// Helper function to check if a value is empty and handle Option types using a closure
/// It can handle `Option<Option<T>>`, `Option<T>`, and T types.
/// It returns true if the value is empty or None, otherwise it returns false.
pub fn is_empty_or_none(value: &dyn Any) -> bool {
    macro_rules! check_type {
        ($ty:ty, $predicate:expr) => {{
            if let Some(opt) = value.downcast_ref::<Option<Option<$ty>>>() {
                return opt.as_ref().map_or(true, |v| v.as_ref().map_or(true, $predicate));
            }

            if let Some(opt) = value.downcast_ref::<Option<$ty>>() {
                return opt.as_ref().map_or(true, $predicate);
            }

            if let Some(v) = value.downcast_ref::<$ty>() {
                return $predicate(v);
            }
        }};
    }

    check_type!(String, |s: &String| s.is_empty() || s.eq_ignore_ascii_case("null"));
    check_type!(&str, |s: &&str| s.is_empty() || s.eq_ignore_ascii_case("null"));
    check_type!(Vec<u8>, |b: &Vec<u8>| b.is_empty());
    check_type!(&[u8], |b: &&[u8]| b.is_empty());

    if let Some(opt) = value.downcast_ref::<Option<()>>() {
        return opt.is_none();
    }

    false
}

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

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
/// Calculates the maximum, minimum, and warmup connection limits based on the provided percentage.
pub fn db_connect_limits(percentage: Option<u32>) -> (u32, u32, u32) {
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(10, num_cpus * 2);
    let min_connections = num_cpus / 2;
    let warmup_connections = percentage.map_or(0, |perc| {
        (max_connections as f32 * (perc as f32 / 100.0)).ceil() as u32
    });
    
    (max_connections, min_connections, warmup_connections)
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
/// Creates a dynamic query function.
pub fn dyn_query<'a, 'b, F, Q>(query_fn: F) -> Option<Box<dyn Fn(&mut Q) + Send + 'b>>
where
    F: Fn(&mut Q) + Send + 'a,
    'a: 'b,
{
    Some(Box::new(query_fn))
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
/// Creates an empty query function.
pub fn empty_query<'a, 'b, Q>() -> Option<Box<dyn Fn(&mut Q) + Send + 'b>> {
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_option() {
        let opt_opt = Some(Some("Hello".to_string()));
        let opt = Some("World".to_string());

        assert_eq!(unwrap_option(&opt_opt), Some(&"Hello".to_string()));
        assert_eq!(unwrap_option(&opt), Some(&"World".to_string()));
    }

    #[test]
    fn test_check_empty_or_none() {
        let str = "Hello";
        let opt_str = Some("World".to_string());
        let opt_none: Option<String> = None;
        let empty_str = "";
        let empty_opt_str = Some("".to_string());
        let empty_opt_none: Option<String> = None;
        let empty_vec:Vec<u8> = vec![];

        assert!(!is_empty_or_none(&str));
        assert!(!is_empty_or_none(&opt_str));
        assert!(is_empty_or_none(&opt_none));
        assert!(is_empty_or_none(&empty_str));
        assert!(is_empty_or_none(&empty_opt_str));
        assert!(is_empty_or_none(&empty_opt_none)); 
        assert!(is_empty_or_none(&empty_vec));       
    }

    #[test]
    fn test_replace_placeholders() {
        let sql = "SELECT * FROM users WHERE id = ? AND name = ?";
        let new_sql = replace_placeholders(sql);
        assert_eq!(new_sql, "SELECT * FROM users WHERE id = $1 AND name = $2");
    }
    
}
