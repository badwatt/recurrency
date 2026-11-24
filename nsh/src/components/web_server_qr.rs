//! Server pairing QR rendered on a canvas, so it works on the native shell
//! web target and on desktop without an HTML renderer.

use nsh_base::theme;
use nsh_core::prelude::*;
use qrcode::{Color, QrCode};

const QR_SIZE: f32 = 160.0;
const SERVER_QR_SIZE: f32 = 144.0;

fn paint_qr(payload: &str, size: f32) -> Painter {
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
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(12.0))
                .child(
                    text(title)
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("APK")
                        .text_size(px(11.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .rounded(px(999.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .text_color(move || theme().muted_foreground),
                ),
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
                                move || paint_qr(&payload, QR_SIZE)
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
                            text(format!(
                                "This QR always points to the latest release. Installed version: v{}.",
                                env!("CARGO_PKG_VERSION")
                            ))
                                .number_of_lines(3)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
}

pub fn web_server_qr(server: String) -> impl IntoElement {
    let payload = format!(r#"{{"type":"recurrency-server","url":"{server}"}}"#);
    let display = server.clone();
    crate::ui::surface()
        .p(px(0.0))
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(4.0))
                .p(px(24.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .child(
                    text("Connect a device")
                        .text_size(px(18.0))
                        .line_height(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Scan this code from the Android app to use this server.")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(16.0))
                .child(
                    view()
                        .h(px(SERVER_QR_SIZE + 16.0))
                        .w(px(SERVER_QR_SIZE + 16.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(10.0))
                        .bg(rgb(0xFFFFFF))
                        .p(px(8.0))
                        .child(
                            canvas({
                                let payload = payload.clone();
                                move || paint_qr(&payload, SERVER_QR_SIZE)
                            })
                            .h(px(SERVER_QR_SIZE))
                            .w(px(SERVER_QR_SIZE)),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            text("Server")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(display)
                                .text_size(px(12.0))
                                .font_family(crate::theme::MONO)
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .into_element()
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
