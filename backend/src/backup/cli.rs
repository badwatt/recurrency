//! Scheduled PostgreSQL logical backups for the all-in-one container.
//!
//! `recurrency backup` uses the PostgreSQL client binaries that ship with the
//! image. It validates every dump, optionally restores it into a throwaway
//! database, and prunes files older than the configured retention window.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const DEFAULT_BACKUP_DIR: &str = "/var/lib/recurrency/backups";
const DEFAULT_RETENTION_DAYS: u64 = 7;
const BACKUP_PREFIX: &str = "recurrency-";
const BACKUP_SUFFIX: &str = ".dump";
const SECONDS_PER_DAY: u64 = 86_400;

struct BackupConfig {
    output_dir: PathBuf,
    retention_days: u64,
    restore_test: bool,
    admin_url: Option<String>,
}

impl BackupConfig {
    fn from_env() -> Self {
        Self {
            output_dir: std::env::var("BACKUP_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(DEFAULT_BACKUP_DIR)),
            retention_days: parse_retention_days(
                std::env::var("BACKUP_RETENTION_DAYS").ok().as_deref(),
            ),
            restore_test: parse_bool(std::env::var("BACKUP_RESTORE_TEST").ok().as_deref(), true),
            admin_url: std::env::var("DATABASE_ADMIN_URL")
                .ok()
                .filter(|url| !url.trim().is_empty()),
        }
    }
}

fn parse_retention_days(value: Option<&str>) -> u64 {
    value
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|days| *days > 0)
        .unwrap_or(DEFAULT_RETENTION_DAYS)
}

fn parse_bool(value: Option<&str>, default: bool) -> bool {
    match value.map(str::trim).map(str::to_ascii_lowercase) {
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on") => true,
        Some(value) if matches!(value.as_str(), "0" | "false" | "no" | "off") => false,
        _ => default,
    }
}

fn backup_file_name(now: DateTime<Utc>) -> String {
    format!(
        "{}{}{}",
        BACKUP_PREFIX,
        now.format("%Y%m%d-%H%M%S"),
        BACKUP_SUFFIX
    )
}

fn database_url_with_name(url: &str, database: &str) -> Result<String, String> {
    let mut parsed =
        reqwest::Url::parse(url).map_err(|error| format!("invalid database URL: {error}"))?;
    parsed.set_path(database);
    Ok(parsed.into())
}

/// Remove the password from a URL passed to `pg_dump`/`pg_restore`; the
/// password is supplied via PGPASSWORD instead of appearing in process argv.
fn database_url_without_password(url: &str) -> Result<(String, Option<String>), String> {
    let mut parsed =
        reqwest::Url::parse(url).map_err(|error| format!("invalid database URL: {error}"))?;
    let password = parsed.password().map(str::to_string);
    parsed
        .set_password(None)
        .map_err(|_| "could not remove database password".to_string())?;
    Ok((parsed.into(), password))
}

fn is_expired(modified: SystemTime, now: SystemTime, retention_days: u64) -> bool {
    let retention = Duration::from_secs(retention_days.saturating_mul(SECONDS_PER_DAY));
    now.duration_since(modified)
        .map(|age| age > retention)
        .unwrap_or(false)
}

fn is_backup_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(BACKUP_PREFIX) && name.ends_with(BACKUP_SUFFIX))
}

fn run_command(command: &mut Command, description: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = command
        .output()
        .map_err(|error| format!("could not run {description}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("{description} failed: {}", stderr.trim()).into())
}

fn command_with_url(
    program: &str,
    url: &str,
    password: Option<&str>,
) -> Result<Command, Box<dyn std::error::Error>> {
    let (clean_url, _) = database_url_without_password(url)?;
    let mut command = Command::new(program);
    if let Some(password) = password {
        command.env("PGPASSWORD", password);
    }
    command.arg(format!("--dbname={clean_url}"));
    Ok(command)
}

async fn restore_into_temporary_database(
    admin_url: &str,
    dump: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let temporary_database = format!("recurrency_restore_{}", Uuid::new_v4().simple());
    let temporary_url = database_url_with_name(admin_url, &temporary_database)?;
    let (temporary_clean_url, temporary_password) = database_url_without_password(&temporary_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(admin_url)
        .await
        .map_err(|error| format!("could not connect to the restore database: {error}"))?;

    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE DATABASE \"{temporary_database}\""
    )))
    .execute(&admin_pool)
    .await
    .map_err(|error| format!("could not create the restore database: {error}"))?;

    let restore_result = async {
        let mut restore = Command::new("pg_restore");
        if let Some(password) = &temporary_password {
            restore.env("PGPASSWORD", password);
        }
        restore
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--exit-on-error")
            .arg(format!("--dbname={temporary_clean_url}"))
            .arg(dump);
        run_command(&mut restore, "pg_restore")?;

        let restored = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&temporary_url)
            .await
            .map_err(|error| format!("could not open the restored database: {error}"))?;
        let _users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&restored)
            .await
            .map_err(|error| format!("could not read the restored database: {error}"))?;
        restored.close().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    let drop_result = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{temporary_database}\" WITH (FORCE)"
    )))
    .execute(&admin_pool)
    .await;
    admin_pool.close().await;

    restore_result?;
    drop_result.map_err(|error| format!("could not drop the restore database: {error}"))?;
    Ok(())
}

fn prune_old_backups(
    directory: &Path,
    retention_days: u64,
    now: SystemTime,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut removed = 0;
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || !is_backup_file(&path) {
            continue;
        }
        let modified = entry.metadata()?.modified()?;
        if is_expired(modified, now, retention_days) {
            std::fs::remove_file(&path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Create a dump, verify it, optionally restore it, and prune old backups.
pub async fn run(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = BackupConfig::from_env();
    std::fs::create_dir_all(&config.output_dir)?;

    let now = Utc::now();
    let dump_path = config.output_dir.join(backup_file_name(now));
    let (_, dump_password) = database_url_without_password(database_url)?;

    let mut dump = command_with_url("pg_dump", database_url, dump_password.as_deref())?;
    dump.arg("--format=custom")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg(format!("--file={}", dump_path.display()));
    if let Err(error) = run_command(&mut dump, "pg_dump") {
        let _ = std::fs::remove_file(&dump_path);
        return Err(error);
    }

    let mut verify = Command::new("pg_restore");
    verify.arg("--list").arg(&dump_path);
    if let Err(error) = run_command(&mut verify, "pg_restore --list") {
        let _ = std::fs::remove_file(&dump_path);
        return Err(error);
    }

    if config.restore_test {
        if let Some(admin_url) = config.admin_url.as_deref() {
            restore_into_temporary_database(admin_url, &dump_path).await?;
            tracing::info!("Backup restore test passed");
        } else {
            tracing::warn!(
                "DATABASE_ADMIN_URL is not set; backup archive was validated but not fully restored"
            );
        }
    }

    let removed = prune_old_backups(&config.output_dir, config.retention_days, now.into())?;
    tracing::info!(
        "Wrote {}, removed {removed} backup(s) older than {} days",
        dump_path.display(),
        config.retention_days
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-13T12:34:56Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn backup_name_is_sortable_and_versioned() {
        assert_eq!(
            backup_file_name(timestamp()),
            "recurrency-20261113-123456.dump"
        );
    }

    #[test]
    fn database_url_replaces_only_the_database_path() {
        let url = database_url_with_name(
            "postgres://user:secret@localhost:5432/recurrency?sslmode=require",
            "restore_test",
        )
        .unwrap();
        assert_eq!(
            url,
            "postgres://user:secret@localhost:5432/restore_test?sslmode=require"
        );
    }

    #[test]
    fn database_url_can_be_stripped_for_process_arguments() {
        let (url, password) = database_url_without_password(
            "postgres://user:secret@localhost:5432/recurrency?sslmode=require",
        )
        .unwrap();
        assert_eq!(
            url,
            "postgres://user@localhost:5432/recurrency?sslmode=require"
        );
        assert_eq!(password.as_deref(), Some("secret"));
    }

    #[test]
    fn retention_accepts_positive_values_and_rejects_invalid_values() {
        assert_eq!(parse_retention_days(Some("14")), 14);
        assert_eq!(parse_retention_days(Some("0")), DEFAULT_RETENTION_DAYS);
        assert_eq!(parse_retention_days(Some("nope")), DEFAULT_RETENTION_DAYS);
        assert_eq!(parse_retention_days(None), DEFAULT_RETENTION_DAYS);
    }

    #[test]
    fn retention_bool_is_explicit() {
        assert!(parse_bool(Some("true"), false));
        assert!(!parse_bool(Some("off"), true));
        assert!(parse_bool(None, true));
    }

    #[test]
    fn expiry_uses_whole_days() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * SECONDS_PER_DAY);
        assert!(is_expired(
            now - Duration::from_secs(8 * SECONDS_PER_DAY),
            now,
            7
        ));
        assert!(!is_expired(
            now - Duration::from_secs(7 * SECONDS_PER_DAY),
            now,
            7
        ));
        assert!(!is_expired(now + Duration::from_secs(1), now, 7));
    }

    #[test]
    fn only_recurrency_dumps_are_pruned() {
        assert!(is_backup_file(Path::new("recurrency-20261113-123456.dump")));
        assert!(!is_backup_file(Path::new("notes.txt")));
        assert!(!is_backup_file(Path::new("recurrency.dump.tmp")));
    }
}
