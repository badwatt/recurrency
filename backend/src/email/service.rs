use lettre::AsyncTransport;
use serde::Deserialize;

use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub from: String,
    pub app_name: String,
}

pub fn is_configured() -> bool {
    std::env::var("SMTP_HOST").is_ok()
        && std::env::var("SMTP_USER").is_ok()
        && std::env::var("SMTP_PASSWORD").is_ok()
}

pub fn get_config() -> Option<SmtpConfig> {
    if !is_configured() {
        return None;
    }
    Some(SmtpConfig {
        host: std::env::var("SMTP_HOST").unwrap(),
        port: std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap(),
        user: std::env::var("SMTP_USER").unwrap(),
        password: std::env::var("SMTP_PASSWORD").unwrap(),
        from: std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@localhost".to_string()),
        app_name: std::env::var("APP_NAME").unwrap_or_else(|_| "Recurrency".to_string()),
    })
}

fn build_html(email: &str, code: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: system-ui, -apple-system, sans-serif; background: #f9fafb; margin: 0; padding: 20px; }}
        .container {{ max-width: 480px; margin: 0 auto; background: white; border-radius: 12px; padding: 40px; text-align: center; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .logo {{ font-size: 32px; margin-bottom: 24px; }}
        h1 {{ font-size: 24px; color: #111; margin: 0 0 16px; }}
        p {{ color: #666; line-height: 1.6; margin: 0 0 24px; }}
        .code {{ display: inline-block; background: #f3f4f6; border: 2px dashed #d1d5db; border-radius: 8px; padding: 16px 32px; font-size: 32px; font-weight: bold; letter-spacing: 8px; color: #111; font-family: monospace; }}
        .footer {{ margin-top: 32px; font-size: 12px; color: #9ca3af; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">📋</div>
        <h1>Verify your email</h1>
        <p>Enter this code to verify your <strong>{email}</strong> account:</p>
        <div class="code">{code}</div>
        <p class="footer">If you didn't create an account, you can safely ignore this email.</p>
    </div>
</body>
</html>"#,
        email = email,
        code = code
    )
}

pub async fn send_verification_email(to_email: &str, code: &str) -> AppResult<()> {
    let config = get_config().ok_or_else(|| AppError::Internal("SMTP not configured".to_string()))?;

    let from_addr = format!("{} <{}>", config.app_name, config.from);
    let subject = format!("{} - Verify your email", config.app_name);
    let html = build_html(to_email, code);

    let from: lettre::message::Mailbox = from_addr
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid from address: {}", e)))?;
    let to: lettre::message::Mailbox = to_email
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid to address: {}", e)))?;

    let email = lettre::Message::builder()
        .from(from)
        .to(to)
        .subject(&subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(html)
        .map_err(|e| AppError::Internal(format!("Failed to build email: {}", e)))?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        config.user.clone(),
        config.password.clone(),
    );

    let tls_params = lettre::transport::smtp::client::TlsParameters::new(config.host.clone())
        .map_err(|e| AppError::Internal(format!("TLS parameters failed: {}", e)))?;

    let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.host)
        .map_err(|e| AppError::Internal(format!("Failed to create relay: {}", e)))?
        .port(config.port)
        .credentials(creds)
        .tls(lettre::transport::smtp::client::Tls::Wrapper(tls_params))
        .build();

    mailer.send(email).await
        .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;

    tracing::info!("Verification email sent to {}", to_email);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_is_configured() {
        // Should return false without SMTP env vars
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");
        assert!(!is_configured(), "Should not be configured without SMTP env vars");

        // Should return true with SMTP env vars
        std::env::set_var("SMTP_HOST", "smtp.example.com");
        std::env::set_var("SMTP_USER", "user@example.com");
        std::env::set_var("SMTP_PASSWORD", "secret");
        assert!(is_configured(), "Should be configured with SMTP env vars");

        // Clean up
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");
    }

    #[test]
    #[serial]
    fn test_build_html_contains_email_and_code() {
        let html = build_html("test@example.com", "123456");
        assert!(html.contains("test@example.com"));
        assert!(html.contains("123456"));
        assert!(html.contains("Verify your email"));
    }

    #[test]
    #[serial]
    fn test_get_config_none_and_some() {
        // Remove all SMTP env vars
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");
        std::env::remove_var("SMTP_PORT");
        std::env::remove_var("SMTP_FROM");
        std::env::remove_var("APP_NAME");

        // Should return None without env vars
        assert!(get_config().is_none(), "get_config should return None without env vars");

        // Set all env vars
        std::env::set_var("SMTP_HOST", "smtp.gmail.com");
        std::env::set_var("SMTP_USER", "user@gmail.com");
        std::env::set_var("SMTP_PASSWORD", "test_pass_val");
        std::env::set_var("SMTP_PORT", "587");
        std::env::set_var("SMTP_FROM", "noreply@recurrency.app");
        std::env::set_var("APP_NAME", "MyApp");

        // Should return Some with all env vars
        let config = get_config();
        assert!(config.is_some(), "get_config should return Some with all env vars");

        let cfg = config.unwrap();
        assert_eq!(cfg.host, "smtp.gmail.com");
        assert_eq!(cfg.port, 587);
        assert_eq!(cfg.user, "user@gmail.com");
        assert_eq!(cfg.from, "noreply@recurrency.app");
        assert_eq!(cfg.app_name, "MyApp");

        // Clean up
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");
        std::env::remove_var("SMTP_PORT");
        std::env::remove_var("SMTP_FROM");
        std::env::remove_var("APP_NAME");
    }

    #[test]
    #[serial]
    fn test_get_config_default_values() {
        std::env::set_var("SMTP_HOST", "smtp.test.com");
        std::env::set_var("SMTP_USER", "test@test.com");
        std::env::set_var("SMTP_PASSWORD", "pass");
        std::env::remove_var("SMTP_PORT");
        std::env::remove_var("SMTP_FROM");
        std::env::remove_var("APP_NAME");

        let config = get_config().unwrap();
        assert_eq!(config.port, 587); // default
        assert_eq!(config.from, "noreply@localhost"); // default
        assert_eq!(config.app_name, "Recurrency"); // default

        // Clean up
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");
    }

    #[test]
    fn test_build_html_structure() {
        let html = build_html("user@test.com", "654321");
        assert!(html.contains("<html>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("container"));
    }

    #[test]
    fn test_build_html_contains_verify_heading() {
        let html = build_html("test@verify.com", "999999");
        assert!(html.contains("Verify your email"));
    }

    #[test]
    fn test_build_html_code_formatting() {
        let html = build_html("test@code.com", "424242");
        assert!(html.contains("424242"));
        assert!(html.contains("class=\"code\""));
    }

    #[test]
    fn test_build_html_footer_message() {
        let html = build_html("test@footer.com", "111111");
        assert!(html.contains("ignore this email"));
    }

    #[test]
    fn test_smtp_config_clone() {
        let cfg = SmtpConfig {
            host: "smtp.test.com".to_string(),
            port: 587,
            user: "test@test.com".to_string(),
            password: "secret".to_string(),
            from: "noreply@test.com".to_string(),
            app_name: "TestApp".to_string(),
        };
        let cloned = cfg.clone();
        assert_eq!(cfg.host, cloned.host);
        assert_eq!(cfg.port, cloned.port);
    }
}
