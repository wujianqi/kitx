use std::any::Any;

/// 辅助函数，递归展开任意层数的 Option 包装
pub fn unwrap_option<T: Any>(value: &dyn Any) -> Option<&T> {
    if value.is::<Option<T>>() {
        value.downcast_ref::<Option<T>>().and_then(|x| x.as_ref())
    } else if value.is::<T>() {
        value.downcast_ref::<T>()
    } else {
        None
    }
}

/// 辅助函数，判断一个值是否为空，并调用一个闭包函数来处理 Option 类型的情况
pub fn check_empty(value: &dyn Any, chk_none: impl FnOnce(&dyn Any)-> bool) -> bool {
    // 检查空字符串
    if let Some(s) = unwrap_option::<String>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    if let Some(s) = unwrap_option::<&str>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    // 检查空二进制数据
    if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
        return blob.is_empty();
    }
    if let Some(blob) = unwrap_option::<&[u8]>(value) {
        return blob.is_empty();
    }
    // 检查 Option 类型最终包含 None 的情况，并将 value 传递给闭包
    return chk_none(value);
}
