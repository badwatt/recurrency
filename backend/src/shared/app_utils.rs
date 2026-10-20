//! Application utility functions.

use std::env;

const DEFAULT_PORT: u16 = 8080;

/// Parse an APP_PORT value. 0 and invalid values fall back to the default so
/// the server is always bindable.
fn parse_port_value(value: Option<&str>) -> u16 {
    value
        .and_then(|p| p.parse::<u16>().ok())
        .filter(|&p| p != 0)
        .unwrap_or(DEFAULT_PORT)
}

/// Parse the port from APP_PORT env var, defaulting to 8080.
pub fn parse_port() -> u16 {
    parse_port_value(env::var("APP_PORT").ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_default() {
        assert_eq!(parse_port_value(None), DEFAULT_PORT);
    }

    #[test]
    fn parse_port_custom() {
        assert_eq!(parse_port_value(Some("9000")), 9000);
    }

    #[test]
    fn parse_port_invalid() {
        assert_eq!(parse_port_value(Some("not-a-port")), DEFAULT_PORT);
    }

    #[test]
    fn parse_port_zero() {
        assert_eq!(parse_port_value(Some("0")), DEFAULT_PORT);
    }
}