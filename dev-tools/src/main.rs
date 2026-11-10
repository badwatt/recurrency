use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use recurrency_dev::android;
use tokio::net::TcpStream;
use tokio::process::Command as TokioCommand;

const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");
const BACKEND_LOG: &str = "/tmp/recurrency-backend.log";
const FRONTEND_LOG: &str = "/tmp/recurrency-frontend.log";
const DEFAULT_DATABASE_URL: &str =
    "postgres://recurrency:recurrency_dev_password@localhost:5432/recurrency";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let help = args.iter().any(|arg| arg == "--help" || arg == "-h");
    let command = if help {
        "help".to_string()
    } else {
        args.first().cloned().unwrap_or_else(|| "up".to_string())
    };

    let code = match command.as_str() {
        "help" => {
            print_help();
            0
        }
        "up" => run_up(),
        "down" | "stop" => do_down(),
        "db" => do_db(),
        "backend" => do_backend(),
        "frontend" => do_frontend(),
        "css" => do_css(),
        "reset" => do_reset(),
        "coverage" => do_coverage(),
        "build" | "build-all" => do_build(),
        "build-app" => do_build_app(),
        "build-api" => do_build_api(),
        "build-web" => do_build_web(),
        "build-android" => do_build_android(&args[1..]),
        "setup-android" => do_setup_android(),
        "patch-android" => do_patch_android(),
        "build-desktop" => do_build_desktop(),
        "clear" => do_clear(),
        "clear-app" => do_clear_app(),
        "clear-api" => do_clear_api(),
        "clear-web" => do_clear_web(),
        "clear-android" => do_clear_android(),
        "clear-desktop" => do_clear_desktop(),
        "clear-all" => do_clear_all(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_help();
            1
        }
    };

    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn root() -> PathBuf {
    PathBuf::from(MANIFEST)
        .parent()
        .expect("dev-tools must live inside the workspace")
        .to_path_buf()
}

fn has_command(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

/// Commands that need the Rust/Node toolchain run through `nix develop` when
/// the caller is not already inside the dev shell.
fn prefixed(program: &str, args: &[&str]) -> Vec<String> {
    let mut parts = if has_command("dx") {
        Vec::new()
    } else {
        vec![
            "nix".to_string(),
            "develop".to_string(),
            root().to_string_lossy().into_owned(),
            "-c".to_string(),
        ]
    };
    parts.push(program.to_string());
    parts.extend(args.iter().map(|arg| arg.to_string()));
    parts
}

fn envs() -> Vec<(String, String)> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
    let demo = env::var("ENABLE_DEMO").unwrap_or_else(|_| "true".to_string());
    let port = env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string());
    let api = env::var("RECURRENCY_API_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));

    vec![
        ("DATABASE_URL".into(), database_url),
        ("JWT_SECRET".into(), jwt_secret),
        ("ENABLE_DEMO".into(), demo),
        ("APP_PORT".into(), port),
        ("RECURRENCY_API_URL".into(), api),
    ]
}

fn run(cwd: &Path, program: &str, args: &[&str]) -> i32 {
    let parts = prefixed(program, args);
    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..])
        .current_dir(cwd)
        .envs(envs())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    match cmd.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            eprintln!("Failed to run {program}: {err}");
            1
        }
    }
}

fn run_direct(cwd: &Path, program: &str, args: &[&str]) -> i32 {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    match cmd.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            eprintln!("Failed to run {program}: {err}");
            1
        }
    }
}

fn compose_program() -> Option<(String, Vec<String>)> {
    if has_command("docker") {
        return Some((
            "docker".into(),
            vec![
                "compose".into(),
                "-f".into(),
                "docker/docker-compose.dev.yml".into(),
            ],
        ));
    }
    if has_command("podman") {
        return Some((
            "podman".into(),
            vec![
                "compose".into(),
                "-f".into(),
                "docker/docker-compose.dev.yml".into(),
            ],
        ));
    }
    None
}

fn compose(action: &[&str]) -> i32 {
    let Some((program, mut args)) = compose_program() else {
        eprintln!("Neither docker nor podman was found");
        return 1;
    };
    args.extend(action.iter().map(|arg| arg.to_string()));
    run_direct(
        &root(),
        &program,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn kill_stale() {
    for pattern in [
        "target/debug/recurrency-backend",
        "target/release/recurrency-backend",
        "dx serve --platform web --port 1420",
    ] {
        let _ = Command::new("pkill").args(["-f", pattern]).status();
    }
    let _ = Command::new("fuser").args(["-k", "8080/tcp"]).status();
}

fn wait_port_sync(port: u16, seconds: u64) -> bool {
    for _ in 0..seconds {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    false
}

async fn wait_port_async(port: u16, seconds: u64, child: &mut tokio::process::Child) -> bool {
    for _ in 0..seconds {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return true;
        }
        if child.try_wait().ok().flatten().is_some() {
            return false;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    false
}

fn service_command(
    cwd: &Path,
    program: &str,
    args: &[&str],
    log: &str,
) -> io::Result<tokio::process::Child> {
    let parts = prefixed(program, args);
    let file = File::create(log)?;
    let err = file.try_clone()?;
    let mut cmd = TokioCommand::new(&parts[0]);
    cmd.args(&parts[1..])
        .current_dir(cwd)
        .envs(envs())
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(err))
        .kill_on_drop(true);
    cmd.spawn()
}

fn run_up() -> i32 {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map(|runtime| runtime.block_on(up_async()))
        .unwrap_or(1)
}

async fn up_async() -> i32 {
    kill_stale();

    if compose(&["up", "-d"]) != 0 {
        return 1;
    }
    if !wait_port_sync(5432, 30) {
        eprintln!("PostgreSQL did not become ready");
        return 1;
    }

    let mut backend = match service_command(
        &root(),
        "cargo",
        &["run", "-p", "recurrency-backend"],
        BACKEND_LOG,
    ) {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to start backend: {err}");
            return 1;
        }
    };

    if !wait_port_async(8080, 600, &mut backend).await {
        eprintln!("Backend did not become ready. See {BACKEND_LOG}");
        let _ = backend.kill().await;
        compose(&["down"]);
        return 1;
    }

    let frontend_dir = root().join("frontend");
    let mut frontend = match service_command(
        &frontend_dir,
        "dx",
        &["serve", "--platform", "web", "--port", "1420"],
        FRONTEND_LOG,
    ) {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to start frontend: {err}");
            let _ = backend.kill().await;
            compose(&["down"]);
            return 1;
        }
    };

    if !wait_port_async(1420, 600, &mut frontend).await {
        eprintln!("Frontend did not become ready. See {FRONTEND_LOG}");
        let _ = backend.kill().await;
        let _ = frontend.kill().await;
        compose(&["down"]);
        return 1;
    }

    println!();
    println!("Recurrency is running:");
    println!("  Frontend: http://localhost:1420");
    println!("  Backend:  http://localhost:8080");
    println!("  Logs:     {BACKEND_LOG}");
    println!("            {FRONTEND_LOG}");
    println!();
    println!("Press Ctrl+C to stop.");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = backend.wait() => {}
        _ = frontend.wait() => {}
    }

    println!();
    println!("Stopping Recurrency...");
    let _ = backend.kill().await;
    let _ = frontend.kill().await;
    kill_stale();
    compose(&["down"]);
    println!("Stopped.");
    0
}

fn do_down() -> i32 {
    kill_stale();
    let code = compose(&["down"]);
    if code == 0 {
        println!("All stopped.");
    }
    code
}

fn do_db() -> i32 {
    if compose(&["up", "-d"]) != 0 || !wait_port_sync(5432, 30) {
        eprintln!("PostgreSQL did not become ready");
        return 1;
    }
    println!("PostgreSQL running on localhost:5432.");
    0
}

fn do_backend() -> i32 {
    run(&root(), "cargo", &["run", "-p", "recurrency-backend"])
}

fn do_frontend() -> i32 {
    run(
        &root().join("frontend"),
        "dx",
        &["serve", "--platform", "web", "--port", "1420"],
    )
}

fn do_css() -> i32 {
    let frontend = root().join("frontend");
    if has_command("npx") {
        run_direct(
            &frontend,
            "npx",
            &[
                "tailwindcss",
                "-i",
                "assets/styles/tailwind.src.css",
                "-o",
                "assets/styles/tailwind.css",
                "--minify",
            ],
        )
    } else {
        run_direct(&frontend, "npm", &["run", "build:css"])
    }
}

fn do_reset() -> i32 {
    println!("WARNING: this deletes all local database data.");
    print!("Confirm [y/N]: ");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() || !matches!(answer.trim(), "y" | "Y") {
        println!("Cancelled.");
        return 0;
    }

    kill_stale();
    compose(&["down", "-v"]);
    if compose(&["up", "-d"]) != 0 || !wait_port_sync(5432, 30) {
        eprintln!("PostgreSQL did not become ready");
        return 1;
    }
    println!("Database reset. Start the backend (cargo up or cargo backend) to run migrations.");
    0
}

fn do_coverage() -> i32 {
    let backend = run(
        &root().join("backend"),
        "cargo",
        &["llvm-cov", "test", "--lib"],
    );
    if backend != 0 {
        return backend;
    }
    run(&root().join("frontend"), "cargo", &["llvm-cov", "test"])
}

fn do_build() -> i32 {
    let backend = run(&root().join("backend"), "cargo", &["build", "--release"]);
    if backend != 0 {
        return backend;
    }
    let web = run(
        &root().join("frontend"),
        "dx",
        &["bundle", "--platform", "web", "--release"],
    );
    if web != 0 {
        return web;
    }
    let _ = run(
        &root().join("frontend"),
        "dx",
        &[
            "bundle",
            "--platform",
            "desktop",
            "--release",
            "--out-dir",
            "../target/bin/desktop",
        ],
    );
    println!("Build complete: backend + web + desktop.");
    0
}

fn copy_backend_binary() -> i32 {
    let bin = root().join("target/bin");
    let _ = fs::create_dir_all(&bin);
    let source = root().join("backend/target/release/recurrency-backend");
    let destination = bin.join("recurrency-aio");
    match fs::copy(&source, &destination) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Failed to copy backend binary: {err}");
            1
        }
    }
}

fn do_build_app() -> i32 {
    let web = run(
        &root().join("frontend"),
        "dx",
        &["bundle", "--platform", "web", "--release"],
    );
    if web != 0 {
        return web;
    }
    let backend = run(&root().join("backend"), "cargo", &["build", "--release"]);
    if backend != 0 {
        return backend;
    }
    if copy_backend_binary() != 0 {
        return 1;
    }
    println!("Built target/bin/recurrency-aio.");
    0
}

fn do_build_api() -> i32 {
    let backend = run(&root().join("backend"), "cargo", &["build", "--release"]);
    if backend != 0 {
        return backend;
    }
    if copy_backend_binary() != 0 {
        return 1;
    }
    println!("Built target/bin/recurrency-aio.");
    0
}

fn do_build_web() -> i32 {
    let code = run(
        &root().join("frontend"),
        "dx",
        &["bundle", "--platform", "web", "--release"],
    );
    if code == 0 {
        println!("Built frontend/dist/.");
    }
    code
}

fn do_build_android(args: &[String]) -> i32 {
    let mut out_dir = root().join("target/bin/android");
    let mut index = 0;
    while index < args.len() {
        if let Some(value) = args[index].strip_prefix("--out-dir=") {
            out_dir = PathBuf::from(value);
            index += 1;
        } else if args[index] == "--out-dir" {
            if let Some(value) = args.get(index + 1) {
                out_dir = PathBuf::from(value);
                index += 2;
            } else {
                eprintln!("--out-dir needs a value");
                return 1;
            }
        } else {
            index += 1;
        }
    }
    if out_dir.is_relative() {
        out_dir = root().join(out_dir);
    }
    match android::build(&root(), &out_dir) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn do_setup_android() -> i32 {
    match android::setup(&root()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn do_patch_android() -> i32 {
    match android::patch(&root()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn do_build_desktop() -> i32 {
    let code = run(
        &root().join("frontend"),
        "dx",
        &[
            "bundle",
            "--platform",
            "desktop",
            "--release",
            "--out-dir",
            "../target/bin/desktop",
        ],
    );
    if code == 0 {
        println!("Built target/bin/desktop/.");
    }
    code
}

fn remove(path: PathBuf) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn do_clear() -> i32 {
    remove(root().join("backend/target/release/recurrency-backend"));
    remove(root().join("frontend/dist"));
    remove(root().join("target/bin/android"));
    remove(root().join("target/bin/desktop"));
    remove(root().join("frontend/target"));
    println!("Build artifacts cleared.");
    0
}

fn do_clear_app() -> i32 {
    remove(root().join("target/bin/recurrency-aio"));
    remove(root().join("frontend/dist"));
    println!("API + web artifacts cleared.");
    0
}

fn do_clear_api() -> i32 {
    remove(root().join("target/bin/recurrency-aio"));
    remove(root().join("backend/target/release/recurrency-backend"));
    println!("API artifacts cleared.");
    0
}

fn do_clear_web() -> i32 {
    remove(root().join("frontend/dist"));
    println!("Web artifacts cleared.");
    0
}

fn do_clear_android() -> i32 {
    remove(root().join("target/bin/android"));
    remove(root().join("frontend/target/dx/recurrency-frontend/release/android"));
    println!("Android build cleaned.");
    0
}

fn do_clear_desktop() -> i32 {
    remove(root().join("target/bin/desktop"));
    println!("Desktop artifacts cleared.");
    0
}

fn do_clear_all() -> i32 {
    let _ = do_clear();
    let _ = do_clear_android();
    let _ = do_clear_desktop();
    0
}

fn print_help() {
    println!(
        "Usage: cargo <command>

  up            Start PostgreSQL + backend + frontend
  down          Stop everything
  db            Start PostgreSQL only
  backend       Start backend only (port 8080)
  frontend      Start frontend only (port 1420)
  css           Build Tailwind CSS
  reset         Reset the local database (asks for confirmation)
  coverage      Run llvm-cov for backend and frontend
  build-all     Build backend + web + desktop
  build-app     Build web + backend into target/bin/recurrency-aio
  build-api     Build backend into target/bin/recurrency-aio
  build-web     Build the web frontend
  build-android Build the local Android APK
  setup-android Install Android SDK/NDK
  patch-android Patch an already-generated Android project
  build-desktop Build the desktop app
  clear         Clear build artifacts
  clear-app     Clear API + web artifacts
  clear-api     Clear API artifacts
  clear-web     Clear web artifacts
  clear-android Clear Android artifacts
  clear-desktop Clear desktop artifacts
  clear-all     Clear everything

Also: cargo test-all, cargo fmt-all, cargo clippy-all, cargo dev-help."
    );
}
