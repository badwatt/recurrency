use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Check APP_VERSION env var first (set by Docker build args)
    let version = env::var("APP_VERSION").unwrap_or_else(|_| {
        // Fallback: read VERSION file from repo root
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
