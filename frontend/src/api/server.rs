//! Validation for a Recurrency server URL and for QR payloads.

use crate::state::app_mode::normalize_server_url;
use serde::Deserialize;

const INFO_PATH: &str = "/api/info";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ServerInfo {
    name: String,
}

/// Accept either a bare URL or a JSON QR payload such as
/// `{"type":"recurrency-server","url":"https://..."}`.
pub fn parse_qr_payload(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("The QR code is empty".to_string());
    }
    let candidate = if trimmed.starts_with('{') {
        let value: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|_| "The QR code does not contain valid JSON".to_string())?;
        value
            .get("url")
            .or_else(|| value.get("server"))
            .and_then(|value| value.as_str())
            .ok_or_else(|| "The QR code does not contain a server URL".to_string())?
            .to_string()
    } else {
        trimmed.to_string()
    };
    normalize_server_url(&candidate)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_info(url: &str) -> Option<ServerInfo> {
    let response = crate::api::client::get_client()
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json::<ServerInfo>().await.ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_info(url: &str) -> Option<ServerInfo> {
    let url = url.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = {
            let request = ureq::get(&url)
                .header("Accept", "application/json")
                .config()
                .timeout_global(Some(std::time::Duration::from_secs(8)))
                .http_status_as_error(false)
                .build();
            request.call().ok().and_then(|mut response| {
                if !response.status().is_success() {
                    return None;
                }
                response.body_mut().read_json::<ServerInfo>().ok()
            })
        };
        let _ = tx.send(result);
    });
    tokio::time::timeout(std::time::Duration::from_secs(10), rx)
        .await
        .ok()?
        .ok()?
}

/// Validate syntax and prove that the target identifies as Recurrency.
pub async fn validate_server(input: &str) -> Result<String, String> {
    let normalized = normalize_server_url(input)?;
    let info_url = format!("{normalized}{INFO_PATH}");
    let Some(info) = fetch_info(&info_url).await else {
        return Err("Could not reach a Recurrency server at that address".to_string());
    };
    if info.name != "recurrency" {
        return Err("That address is not a Recurrency server".to_string());
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_accepts_a_plain_url() {
        assert_eq!(
            parse_qr_payload("https://recurrency.example.com/").unwrap(),
            "https://recurrency.example.com"
        );
    }

    #[test]
    fn qr_accepts_json_url() {
        assert_eq!(
            parse_qr_payload(r#"{"type":"recurrency-server","url":"https://r.example.com"}"#)
                .unwrap(),
            "https://r.example.com"
        );
    }

    #[test]
    fn qr_rejects_json_without_url() {
        assert!(parse_qr_payload(r#"{"type":"other"}"#).is_err());
    }

    #[test]
    fn qr_rejects_empty_payload() {
        assert!(parse_qr_payload("   ").is_err());
    }

    #[test]
    fn server_info_is_strict() {
        let info: ServerInfo = serde_json::from_str(r#"{"name":"recurrency"}"#).unwrap();
        assert_eq!(info.name, "recurrency");
    }
    #[tokio::test]
    async fn validate_server_accepts_a_recurrency_info_response() {
        let (base, server) =
            crate::api::test_support::serve_once("200 OK", r#"{"name":"recurrency"}"#);
        let result = validate_server(&base).await.unwrap();
        server.join().unwrap();
        assert_eq!(result, base);
    }

    #[tokio::test]
    async fn validate_server_rejects_other_servers() {
        let (base, server) =
            crate::api::test_support::serve_once("200 OK", r#"{"name":"not-recurrency"}"#);
        let error = validate_server(&base).await.unwrap_err();
        server.join().unwrap();
        assert!(error.contains("not a Recurrency server"));
    }

    #[tokio::test]
    async fn validate_server_reports_unreachable_addresses() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let error = validate_server(&format!("http://{address}"))
            .await
            .unwrap_err();
        assert!(error.contains("Could not reach"));
    }

    #[test]
    fn qr_rejects_invalid_json_and_empty_values() {
        assert!(parse_qr_payload("{not-json").is_err());
        assert!(parse_qr_payload(r#"{"server":""}"#).is_err());
        assert_eq!(
            parse_qr_payload(r#"{"server":"https://r.example.com"}"#).unwrap(),
            "https://r.example.com"
        );
    }
}
