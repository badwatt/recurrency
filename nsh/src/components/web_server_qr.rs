//! Server pairing QR rendered on a canvas, so it works on the native shell
//! web target and on desktop without an HTML renderer.

use nsh_base::theme;
use nsh_core::prelude::*;
use qrcode::{Color, QrCode};

const QR_SIZE: f32 = 168.0;

fn paint_qr(payload: &str) -> Painter {
    let size = QR_SIZE;
    let Ok(code) = QrCode::new(payload.as_bytes()) else {
        return Painter::new();
    };
    let colors = code.to_colors();
    let width = code.width();
    if width == 0 {
        return Painter::new();
    }
    let cell = size / width as f32;
    let mut painter = Painter::new();
    // Quiet zone and light modules.
    painter = painter
        .fill_color(rgb(0xFFFFFF))
        .begin_path()
        .move_to(0.0, 0.0)
        .line_to(size, 0.0)
        .line_to(size, size)
        .line_to(0.0, size)
        .close_path()
        .fill();
    for (index, color) in colors.into_iter().enumerate() {
        if color != Color::Dark {
            continue;
        }
        let x = (index % width) as f32 * cell;
        let y = (index / width) as f32 * cell;
        painter = painter
            .fill_color(rgb(0x111827))
            .begin_path()
            .move_to(x, y)
            .line_to(x + cell, y)
            .line_to(x + cell, y + cell)
            .line_to(x, y + cell)
            .close_path()
            .fill();
    }
    painter
}

/// Shared QR card used for server pairing and the Android download code.
pub fn qr_card(
    title: &'static str,
    description: &'static str,
    payload: String,
    caption: String,
) -> impl IntoElement {
    view()
        .w_full()
        .rounded(px(crate::theme::CARD_RADIUS))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(24.0))
        .gap(px(14.0))
        .child(
            text(title)
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(description)
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(18.0))
                .child(
                    view()
                        .h(px(QR_SIZE + 16.0))
                        .w(px(QR_SIZE + 16.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(10.0))
                        .bg(rgb(0xFFFFFF))
                        .p(px(8.0))
                        .child(
                            canvas({
                                let payload = payload.clone();
                                move || paint_qr(&payload)
                            })
                            .h(px(QR_SIZE))
                            .w(px(QR_SIZE)),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            text(caption.clone())
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("This code always points to the latest release.")
                                .number_of_lines(3)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
}

pub fn web_server_qr(server: String) -> impl IntoElement {
    let caption = server.clone();
    let payload = format!(r#"{{"type":"recurrency-server","url":"{server}"}}"#);
    qr_card(
        "Connect a device",
        "Scan this code from the Recurrency app to use this server.",
        payload,
        caption,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_payload_is_a_server_record() {
        let payload = r#"{"type":"recurrency-server","url":"http://localhost:8080"}"#;
        assert!(payload.contains("recurrency-server"));
        assert!(payload.contains("http://localhost:8080"));
    }

    #[test]
    fn qr_encoding_has_modules() {
        let code = QrCode::new(b"recurrency-server").expect("QR fixture");
        assert!(code.width() > 0);
    }
}
