//! Small command-line surface for the standalone all-in-one binary.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Serve,
    Migrate,
    Backup,
    Admin,
    Doctor,
    Help,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAction {
    List,
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminArgs {
    pub action: AdminAction,
    pub target: Option<String>,
    pub email: Option<String>,
    pub new_email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
    pub port: Option<u16>,
    pub database_url: Option<String>,
    pub backup_dir: Option<PathBuf>,
    pub enable_web: Option<bool>,
    pub admin: Option<AdminArgs>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            command: Command::Serve,
            port: None,
            database_url: None,
            backup_dir: None,
            enable_web: None,
            admin: None,
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
            "backup" => Command::Backup,
            "admin" => Command::Admin,
            "doctor" => Command::Doctor,
            "help" => Command::Help,
            "version" => Command::Version,
            other => return Err(format!("unknown command: {other}")),
        };
        args.next();
        if cli.command == Command::Admin {
            let action = args.next().ok_or_else(|| {
                "admin needs an action: list, create, update or delete".to_string()
            })?;
            let action = match action.as_str() {
                "list" => AdminAction::List,
                "create" => AdminAction::Create,
                "update" => AdminAction::Update,
                "delete" => AdminAction::Delete,
                other => return Err(format!("unknown admin action: {other}")),
            };
            cli.admin = Some(AdminArgs {
                action,
                target: None,
                email: None,
                new_email: None,
                name: None,
                password: None,
                is_admin: None,
            });
        }
    }

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => cli.command = Command::Help,
            "--version" | "-V" => cli.command = Command::Version,
            "--port" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--port needs a value".to_string())?;
                cli.port = Some(
                    value
                        .parse()
                        .map_err(|_| format!("invalid port: {value}"))?,
                );
            }
            "--database-url" => {
                cli.database_url = Some(
                    args.next()
                        .ok_or_else(|| "--database-url needs a value".to_string())?,
                );
            }
            "--backup-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--backup-dir needs a value".to_string())?;
                cli.backup_dir = Some(PathBuf::from(value));
            }
            "--email" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--email needs a value".to_string())?;
                if let Some(admin) = &mut cli.admin {
                    admin.email = Some(value);
                } else {
                    return Err("--email is only valid with admin".to_string());
                }
            }
            "--new-email" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--new-email needs a value".to_string())?;
                if let Some(admin) = &mut cli.admin {
                    admin.new_email = Some(value);
                } else {
                    return Err("--new-email is only valid with admin".to_string());
                }
            }
            "--name" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--name needs a value".to_string())?;
                if let Some(admin) = &mut cli.admin {
                    admin.name = Some(value);
                } else {
                    return Err("--name is only valid with admin".to_string());
                }
            }
            "--password" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--password needs a value".to_string())?;
                if let Some(admin) = &mut cli.admin {
                    admin.password = Some(value);
                } else {
                    return Err("--password is only valid with admin".to_string());
                }
            }
            "--admin" => {
                if let Some(admin) = &mut cli.admin {
                    admin.is_admin = Some(true);
                } else {
                    return Err("--admin is only valid with admin".to_string());
                }
            }
            "--no-admin" => {
                if let Some(admin) = &mut cli.admin {
                    admin.is_admin = Some(false);
                } else {
                    return Err("--no-admin is only valid with admin".to_string());
                }
            }
            "--no-web" => cli.enable_web = Some(false),
            "--enable-web" => cli.enable_web = Some(true),
            other
                if cli.command == Command::Admin
                    && !other.starts_with('-')
                    && cli
                        .admin
                        .as_ref()
                        .is_some_and(|admin| admin.target.is_none()) =>
            {
                if let Some(admin) = &mut cli.admin {
                    admin.target = Some(other.to_string());
                }
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }
    Ok(cli)
}

pub fn usage() -> &'static str {
    "Recurrency all-in-one\n\n\
     Usage:\n\
       recurrency [serve] [options]\n\
       recurrency migrate [--database-url URL]\n\
       recurrency backup [--database-url URL] [--backup-dir DIR]\n\
       recurrency doctor\n\n\
     Options:\n\
       --port PORT             API port (default: APP_PORT or 8080)\n\
       --database-url URL      PostgreSQL connection string; omit to use the\n\
                               local database under RECURRENCY_DATA_DIR\n\
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
        assert!(cli.backup_dir.is_none());
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
        let backup = parse(args(&["backup", "--backup-dir", "/tmp/backups"])).unwrap();
        assert_eq!(backup.command, Command::Backup);
        assert_eq!(
            backup.backup_dir.as_deref(),
            Some(std::path::Path::new("/tmp/backups"))
        );
    }

    #[test]
    fn parses_commands_and_help() {
        assert_eq!(parse(args(&["migrate"])).unwrap().command, Command::Migrate);
        assert_eq!(parse(args(&["backup"])).unwrap().command, Command::Backup);
        assert_eq!(parse(args(&["doctor"])).unwrap().command, Command::Doctor);
        let admin = parse(args(&["admin", "list"])).unwrap();
        assert_eq!(admin.command, Command::Admin);
        assert_eq!(admin.admin.unwrap().action, AdminAction::List);
        assert_eq!(parse(args(&["--help"])).unwrap().command, Command::Help);
        assert_eq!(
            parse(args(&["--version"])).unwrap().command,
            Command::Version
        );
    }

    #[test]
    fn parses_admin_crud_arguments() {
        let create = parse(args(&[
            "admin",
            "create",
            "--email",
            "user@example.com",
            "--password",
            "password123",
            "--name",
            "User",
            "--admin",
        ]))
        .unwrap();
        let create = create.admin.unwrap();
        assert_eq!(create.action, AdminAction::Create);
        assert_eq!(create.email.as_deref(), Some("user@example.com"));
        assert_eq!(create.is_admin, Some(true));

        let update = parse(args(&[
            "admin",
            "update",
            "user@example.com",
            "--new-email",
            "new@example.com",
            "--name",
            "New",
            "--no-admin",
        ]))
        .unwrap();
        let update = update.admin.unwrap();
        assert_eq!(update.action, AdminAction::Update);
        assert_eq!(update.target.as_deref(), Some("user@example.com"));
        assert_eq!(update.new_email.as_deref(), Some("new@example.com"));
        assert_eq!(update.is_admin, Some(false));

        let delete = parse(args(&["admin", "delete", "user@example.com"])).unwrap();
        assert_eq!(
            delete.admin.unwrap().target.as_deref(),
            Some("user@example.com")
        );
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse(args(&["--port", "nope"])).is_err());
        assert!(parse(args(&["--wat"])).is_err());
        assert!(parse(args(&["--port"])).is_err());
    }
}
