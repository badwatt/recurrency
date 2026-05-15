//! Build script: inject APP_VERSION from VERSION file.
//! Frontend is no longer embedded — served by separate container/nginx.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // --- Inject APP_VERSION from VERSION file ---
    let version = env::var("APP_VERSION").unwrap_or_else(|_| {
        let version_path = Path::new("../VERSION");
        fs::read_to_string(version_path)
            .unwrap_or_else(|_| "dev".into())
            .trim()
            .to_string()
    });
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rerun-if-changed=../VERSION");
    println!("cargo:rerun-if-env-changed=APP_VERSION");
}