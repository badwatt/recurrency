//! Test stub for embedded module.
//! Used in test builds to avoid include_dir! compile error
//! when frontend/dist doesn't exist.

pub fn has_embedded() -> bool {
    false
}

pub fn init() -> bool {
    false
}
