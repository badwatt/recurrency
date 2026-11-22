//! Session state, server configuration and the REST calls used by the native
//! login flow.

use nsh_core::{
    prelude::*,
    storage::Storage,
};
use serde::{Deserialize, Serialize};

const TOKEN_KEY: &str = "recurrency.token";
const USER_KEY: &str = "recurrency.user";
const SERVER_KEY: &str = "recurrency.server";
pub const DEFAULT_SERVER: &str = "http://localhost:8080";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    pub token: String,
    pub user: AuthUser,
    #[serde(default)]
    pub requires_verification: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Session {
    Loading,
    SignedOut,
    SignedIn(AuthSession),
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct DemoStatus {
    #[serde(default)]
    enabled: bool,
}

#[derive(Deserialize)]
struct DemoCredentials {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterRequest<'a> {
    email: &'a str,
    password: &'a str,
    name: &'a str,
}

pub fn stored_server() -> String {
    Storage::get_item(SERVER_KEY)
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_SERVER.to_string())
}

pub fn store_server(server: &str) {
    let _ = Storage::set_item(SERVER_KEY, server.trim().trim_end_matches('/'));
}

pub fn store_session(session: &AuthSession) {
    let _ = Storage::set_item(TOKEN_KEY, &session.token);
    if let Ok(user) = serde_json::to_string(&session.user) {
        let _ = Storage::set_item(USER_KEY, user);
    }
}

pub fn clear_session() {
    Storage::remove_item(TOKEN_KEY);
    Storage::remove_item(USER_KEY);
}

pub fn has_stored_session() -> bool {
    Storage::get_item(TOKEN_KEY).is_some_and(|token| !token.is_empty())
        && Storage::get_item(USER_KEY).is_some()
}

pub async fn restore_session() -> Session {
    let Some(token) = Storage::get_item(TOKEN_KEY).filter(|token| !token.is_empty()) else {
        return Session::SignedOut;
    };
    let Some(user_json) = Storage::get_item(USER_KEY) else {
        clear_session();
        return Session::SignedOut;
    };
    let Ok(user) = serde_json::from_str::<AuthUser>(&user_json) else {
        clear_session();
        return Session::SignedOut;
    };
    // Revalidate against the server. A token that no longer verifies is
    // discarded instead of showing a stale dashboard.
    let request = HttpRequest::get(format!("{}/api/settings", stored_server()))
        .header("Authorization", format!("Bearer {token}"));
    match fetch(request).await {
        Ok(response) if response.ok() => Session::SignedIn(AuthSession {
            token,
            user,
            requires_verification: false,
        }),
        Ok(_) | Err(_) => {
            clear_session();
            Session::SignedOut
        }
    }
}

pub async fn login(server: &str, email: &str, password: &str) -> Result<AuthSession, String> {
    let server = server.trim().trim_end_matches('/');
    let body = serde_json::to_string(&LoginRequest { email, password })
        .map_err(|error| error.to_string())?;
    let request = HttpRequest::post(format!("{server}/api/auth/login"), body)
        .header("Content-Type", "application/json");
    let response = fetch(request).await?;
    if !response.ok() {
        let status = response.status();
        let body = response.text();
        return Err(error_message(status, &body));
    }
    let body = response.text();
    let session = serde_json::from_str::<AuthSession>(&body)
        .map_err(|_| "The server returned an unexpected response".to_string())?;
    store_server(server);
    store_session(&session);
    Ok(session)
}

pub async fn demo_enabled(server: &str) -> bool {
    let server = server.trim().trim_end_matches('/');
    match fetch(format!("{server}/api/demo/status")).await {
        Ok(response) if response.ok() => serde_json::from_str::<DemoStatus>(&response.text())
            .map(|status| status.enabled)
            .unwrap_or(false),
        _ => false,
    }
}

pub async fn demo_login(server: &str) -> Result<AuthSession, String> {
    let server = server.trim().trim_end_matches('/');
    let response = fetch(format!("{server}/api/demo/init")).await?;
    if !response.ok() {
        return Err(error_message(response.status(), &response.text()));
    }
    let credentials: DemoCredentials = serde_json::from_str(&response.text())
        .map_err(|_| "The server returned an unexpected response".to_string())?;
    let session = login(server, &credentials.email, &credentials.password).await?;
    let reset = HttpRequest::post(format!("{server}/api/demo/reset"), "{}")
        .header("Authorization", format!("Bearer {}", session.token))
        .header("Content-Type", "application/json");
    let _ = fetch(reset).await;
    Ok(session)
}

pub async fn register(
    server: &str,
    name: &str,
    email: &str,
    password: &str,
) -> Result<AuthSession, String> {
    let server = server.trim().trim_end_matches('/');
    let body = serde_json::to_string(&RegisterRequest {
        email,
        password,
        name,
    })
    .map_err(|error| error.to_string())?;
    let request = HttpRequest::post(format!("{server}/api/auth/register"), body)
        .header("Content-Type", "application/json");
    let response = fetch(request).await?;
    if !response.ok() {
        let status = response.status();
        let body = response.text();
        return Err(error_message(status, &body));
    }
    let body = response.text();
    let session = serde_json::from_str::<AuthSession>(&body)
        .map_err(|_| "The server returned an unexpected response".to_string())?;
    store_server(server);
    store_session(&session);
    Ok(session)
}

#[derive(Serialize)]
struct VerifyEmailRequest<'a> {
    code: &'a str,
}

pub async fn verify_email(server: &str, token: &str, code: &str) -> Result<AuthUser, String> {
    let server = server.trim().trim_end_matches('/');
    let body = serde_json::to_string(&VerifyEmailRequest { code }).map_err(|error| error.to_string())?;
    let request = HttpRequest::post(format!("{server}/api/auth/verify-email"), body)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json");
    let response = fetch(request).await?;
    if !response.ok() {
        return Err(error_message(response.status(), &response.text()));
    }
    serde_json::from_str::<AuthUser>(&response.text())
        .map_err(|_| "The server returned an unexpected response".to_string())
}

pub async fn resend_verification(server: &str, token: &str) -> Result<(), String> {
    let server = server.trim().trim_end_matches('/');
    let request = HttpRequest::post(format!("{server}/api/auth/resend-verification"), "{}")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json");
    let response = fetch(request).await?;
    if response.ok() {
        Ok(())
    } else {
        Err(error_message(response.status(), &response.text()))
    }
}

fn error_message(status: u16, body: &str) -> String {
    let detail = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .or_else(|| value.get("message"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        });
    match (status, detail) {
        (401, _) => "Email or password is incorrect".to_string(),
        (_, Some(message)) if !message.is_empty() => message,
        _ => format!("The server answered {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_defaults_to_localhost() {
        assert_eq!(DEFAULT_SERVER, "http://localhost:8080");
    }

    #[test]
    fn auth_session_roundtrip() {
        let session = AuthSession {
            token: "token".to_string(),
            user: AuthUser {
                id: "id".to_string(),
                email: "user@example.com".to_string(),
                avatar: None,
                name: Some("User".to_string()),
                currency: "EUR".to_string(),
                verified: true,
                is_admin: false,
            },
            requires_verification: false,
        };
        let json = serde_json::to_string(&session).expect("session serializes");
        let back: AuthSession = serde_json::from_str(&json).expect("session parses");
        assert_eq!(session, back);
    }

    #[test]
    fn error_message_uses_api_error() {
        assert_eq!(
            error_message(400, r#"{"error":"Invalid email"}"#),
            "Invalid email"
        );
        assert_eq!(error_message(401, "{}"), "Email or password is incorrect");
    }
}
