//! Test stub for embedded module.
//! Used in test builds to avoid include_dir! compile error
//! when frontend/dist doesn't exist.

#[allow(dead_code)]
pub fn has_embedded() -> bool {
    false
}

pub fn init() -> bool {
    false
}
