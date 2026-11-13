//! QR generation and Android QR scanning.

use dioxus::prelude::*;

/// Render a compact SVG QR code.
pub fn svg(data: &str) -> Result<String, String> {
    let code = qrcode::QrCode::new(data.as_bytes()).map_err(|error| error.to_string())?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .dark_color(qrcode::render::svg::Color("#111827"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build())
}

/// Open the native Android scanner. The bridge is installed by
/// `cargo build-android`; other targets report the scanner as unavailable.
pub async fn scan() -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        let mut eval = document::eval(
            r#"
            if (!window.RecurrencyQr || !window.RecurrencyQr.scan) {
                dioxus.send('');
                return;
            }
            await new Promise((resolve) => {
                window.__recurrencyQrScanned = (value) => {
                    dioxus.send(value || '');
                    resolve();
                };
                window.RecurrencyQr.scan();
            });
            "#,
        );
        match eval.recv::<String>().await {
            Ok(value) if !value.is_empty() => Ok(value),
            Ok(_) => Err("No QR code was scanned".to_string()),
            Err(_) => Err("Could not open the QR scanner".to_string()),
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        Err("QR scanning is available in the Android app".to_string())
    }
}

pub const fn scanner_available() -> bool {
    cfg!(target_os = "android")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_contains_the_qr_markup() {
        let svg = svg("https://recurrency.example.com").unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
    }

    #[test]
    fn scanner_is_android_only() {
        assert_eq!(scanner_available(), cfg!(target_os = "android"));
    }
}
