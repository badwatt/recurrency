pub mod service;

pub use service::{is_configured, send_text_email, send_text_email_with_config, send_verification_email, SmtpConfig};
