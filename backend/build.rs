//! Build script: compile Dioxus web frontend at compile time.
//! Output embedded in binary - no dist/ folder needed.

use std::env;
use std::path::Path;

fn main() {
    // Only embed when building the binary, not for lib
    let profile = env::var("PROFILE").unwrap_or_default();
    if profile == "test" || profile == "bench" {
        println!("cargo:rerun-if-changed=frontend/src");
        return;
    }

    // Check BUILD_UI flag
    let build_ui = env::var("BUILD_UI").unwrap_or_else(|_| "true".into());
    if build_ui != "true" {
        println!("cargo:rerun-if-changed=frontend/src");
        return;
    }

    let frontend_dir = Path::new("frontend");
    let dist_dir = frontend_dir.join("dist");

    // Run dx bundle if dist doesn't exist or is outdated
    let needs_build = !dist_dir.join("index.html").exists()
        || std::fs::read_dir(&dist_dir).map(|i| i.count()).unwrap_or(0) == 0;

    if needs_build {
        println!("cargo:warning=Frontend dist/ not found. Building web bundle...");
        let status = std::process::Command::new("dx")
            .args(["bundle", "--platform", "web"])
            .current_dir(frontend_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=Frontend built successfully");
            }
            Ok(_) => {
                eprintln!("cargo:warning=Frontend build failed, continuing anyway...");
            }
            Err(e) => {
                eprintln!("cargo:warning=dx command not found: {}. Frontend will be served from external dist/.", e);
            }
        }
    }

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/dist");
}
