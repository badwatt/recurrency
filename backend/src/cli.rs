//! Small command-line surface for the standalone all-in-one binary.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Serve,
    Migrate,
    Doctor,
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
    pub port: Option<u16>,
    pub database_url: Option<String>,
    pub enable_web: Option<bool>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            command: Command::Serve,
            port: None,
            database_url: None,
            enable_web: None,
        }
    }
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut args = args.into_iter().peekable();
    if let Some(first) = args.peek()
        && !first.starts_with('-')
    {
        cli.command = match first.as_str() {
            "serve" => Command::Serve,
            "migrate" => Command::Migrate,
            "doctor" => Command::Doctor,
            "help" => Command::Help,
            "version" => Command::Version,
            other => return Err(format!("unknown command: {other}")),
        };
        args.next();
    }

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => cli.command = Command::Help,
            "--version" | "-V" => cli.command = Command::Version,
            "--port" => {
                let value = args.next().ok_or_else(|| "--port needs a value".to_string())?;
                cli.port = Some(value.parse().map_err(|_| format!("invalid port: {value}"))?);
            }
            "--database-url" => {
                cli.database_url = Some(
                    args.next()
                        .ok_or_else(|| "--database-url needs a value".to_string())?,
                );
            }
            "--no-web" => cli.enable_web = Some(false),
            "--enable-web" => cli.enable_web = Some(true),
            other => return Err(format!("unknown option: {other}")),
        }
    }
    Ok(cli)
}

pub fn usage() -> &'static str {
    "Recurrency all-in-one\n\n\
     Usage:\n\
       recurrency-aio [serve] [options]\n\
       recurrency-aio migrate [--database-url URL]\n\
       recurrency-aio doctor\n\n\
     Options:\n\
       --port PORT             API port (default: APP_PORT or 8080)\n\
       --database-url URL      PostgreSQL connection string\n\
       --no-web                API-only mode; do not serve the embedded web UI\n\
       --enable-web            Serve the embedded web UI\n\
       -h, --help              Show this help\n\
       -V, --version           Show the version"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn defaults_to_serve() {
        let cli = parse(args(&[])).unwrap();
        assert_eq!(cli.command, Command::Serve);
        assert!(cli.port.is_none());
        assert!(cli.enable_web.is_none());
    }

    #[test]
    fn parses_serve_options() {
        let cli = parse(args(&[
            "serve",
            "--port",
            "9090",
            "--database-url",
            "postgres://db/app",
            "--no-web",
        ]))
        .unwrap();
        assert_eq!(cli.command, Command::Serve);
        assert_eq!(cli.port, Some(9090));
        assert_eq!(cli.database_url.as_deref(), Some("postgres://db/app"));
        assert_eq!(cli.enable_web, Some(false));
    }

    #[test]
    fn parses_commands_and_help() {
        assert_eq!(parse(args(&["migrate"])).unwrap().command, Command::Migrate);
        assert_eq!(parse(args(&["doctor"])).unwrap().command, Command::Doctor);
        assert_eq!(parse(args(&["--help"])).unwrap().command, Command::Help);
        assert_eq!(parse(args(&["--version"])).unwrap().command, Command::Version);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse(args(&["--port", "nope"])).is_err());
        assert!(parse(args(&["--wat"])).is_err());
        assert!(parse(args(&["--port"])).is_err());
    }
}
