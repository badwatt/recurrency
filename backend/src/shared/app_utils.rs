//! Application utility functions.

use std::env;

/// Parse the port from APP_PORT env var, defaulting to 8080.
/// Port 0 is treated as invalid (lets OS pick a port, not useful for a server).
pub fn parse_port() -> u16 {
    env::var("APP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .filter(|&p| p != 0)
        .unwrap_or(8080)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::test_env;

    #[test]
    fn parse_port_default() {
        test_env::remove_env("APP_PORT");
        let port = parse_port();
        assert_eq!(port, 8080);
    }

    #[test]
    fn parse_port_from_env_custom() {
        test_env::set_env("APP_PORT", "9000");
        let port = parse_port();
        assert_eq!(port, 9000);
        test_env::remove_env("APP_PORT");
    }

    #[test]
    fn parse_port_from_env_invalid() {
        test_env::set_env("APP_PORT", "not-a-port");
        let port = parse_port();
        assert_eq!(port, 8080);
        test_env::remove_env("APP_PORT");
    }

    #[test]
    fn parse_port_from_env_zero() {
        test_env::set_env("APP_PORT", "0");
        let port = parse_port();
        assert_eq!(port, 8080);
        test_env::remove_env("APP_PORT");
    }
}