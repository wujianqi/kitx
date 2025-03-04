use std::any::Any;

/// Helper function to recursively unwrap any number of Option layers
pub fn unwrap_option<T: Any>(value: &dyn Any) -> Option<&T> {
    if value.is::<Option<T>>() {
        value.downcast_ref::<Option<T>>().and_then(|x| x.as_ref())
    } else if value.is::<T>() {
        value.downcast_ref::<T>()
    } else {
        None
    }
}

/// Helper function to check if a value is empty and handle Option types using a closure
pub fn check_empty(value: &dyn Any, chk_none: impl FnOnce(&dyn Any) -> bool) -> bool {
    // Check for empty string
    if let Some(s) = unwrap_option::<String>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    if let Some(s) = unwrap_option::<&str>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    // Check for empty binary data
    if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
        return blob.is_empty();
    }
    if let Some(blob) = unwrap_option::<&[u8]>(value) {
        return blob.is_empty();
    }
    // Check for Option types that ultimately contain None and pass the value to the closure
    return chk_none(value);
}
