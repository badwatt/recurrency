//! Grid of the 18 palettes. Selecting one applies it immediately, stores it
//! locally and syncs the choice to the server.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn theme_picker(selected: Signal<String>, server: String, token: String) -> impl IntoElement {
    view()
        .w_full()
        .grid()
        .grid_template_columns(SharedString::from("repeat(2, minmax(0, 1fr))"))
        .gap(px(8.0))
        .children(crate::theme::THEMES.iter().map(|definition| {
            let id = definition.id;
            let card_server = server.clone();
            let card_token = token.clone();
            let active = move || selected.get() == id;
            button()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .p(px(10.0))
                .rounded(px(9.0))
                .border(px(1.0))
                .border_color(move || {
                    if active() {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || {
                    if active() {
                        theme().accent
                    } else {
                        theme().background
                    }
                })
                .on_press(move || {
                    crate::theme::apply(id);
                    crate::theme::save(id);
                    selected.set(id.to_string());
                    let server = card_server.clone();
                    let token = card_token.clone();
                    spawn(async move {
                        let _ = crate::api::save_theme(server, token, id).await;
                    });
                })
                .child(
                    text(definition.label)
                        .flex_grow(1.0)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(3.0))
                        .child(
                            view()
                                .h(px(12.0))
                                .w(px(12.0))
                                .rounded(px(3.0))
                                .bg(rgba((definition.background << 8) | 0xff)),
                        )
                        .child(
                            view()
                                .h(px(12.0))
                                .w(px(12.0))
                                .rounded(px(3.0))
                                .bg(rgba((definition.card << 8) | 0xff)),
                        )
                        .child(
                            view()
                                .h(px(12.0))
                                .w(px(12.0))
                                .rounded(px(3.0))
                                .bg(rgba((definition.primary << 8) | 0xff)),
                        ),
                )
                .child(
                    when(active, move || {
                        image(asset!("icons/check.svg"))
                            .size(px(13.0))
                            .tint_color(move || theme().primary)
                            .into_element()
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .into_element()
        }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_theme_appears_in_the_picker() {
        assert_eq!(crate::theme::THEMES.len(), 18);
    }
}

/// Compact popover used by the top-bar paintbrush, matching the Dioxus theme
/// menu: grouped dark/light palettes, swatches and immediate persistence.
pub fn theme_menu() -> AnyElement {
    let selected = signal(crate::theme::saved());
    view()
        .w(px(384.0))
        .p(px(8.0))
        .gap(px(2.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .shadow([BoxShadow {
            offset_y: px(12.0),
            blur: px(32.0),
            color: rgba(0x00000033).into(),
            ..BoxShadow::default()
        }])
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(6.0))
                .child(
                    text("THEME")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(if crate::local::enabled() {
                        "Saved on this device"
                    } else {
                        "Synced across devices"
                    })
                    .text_size(px(10.0))
                    .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            scroll().max_h(px(520.0)).child(
                view().w_full().flex_col().gap(px(2.0)).children(
                    [("Dark", true), ("Light", false)]
                        .into_iter()
                        .map(|(label, is_dark)| {
                            view()
                                .w_full()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    text(label)
                                        .px(px(8.0))
                                        .pt(px(6.0))
                                        .text_size(px(9.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().muted_foreground),
                                )
                                .child(
                                    view()
                                        .w_full()
                                        .grid()
                                        .grid_template_columns(SharedString::from(
                                            "repeat(2, minmax(0, 1fr))",
                                        ))
                                        .gap(px(4.0))
                                        .children(
                                            crate::theme::THEMES
                                                .iter()
                                                .filter(move |item| item.is_dark == is_dark)
                                                .map(|definition| {
                                                    let id = definition.id;
                                                    let active = move || selected.get() == id;
                                                    button()
                                                        .w_full()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(8.0))
                                                        .px(px(8.0))
                                                        .py(px(8.0))
                                                        .rounded(px(6.0))
                                                        .border(px(1.0))
                                                        .border_color(move || {
                                                            if active() {
                                                                theme().primary.opacity(0.4)
                                                            } else {
                                                                theme().border.opacity(0.0)
                                                            }
                                                        })
                                                        .bg(move || {
                                                            if active() {
                                                                theme().accent
                                                            } else {
                                                                theme().card.opacity(0.0)
                                                            }
                                                        })
                                                        .on_press(move || {
                                                            crate::theme::apply(id);
                                                            crate::theme::save(id);
                                                            selected.set(id.to_string());
                                                            crate::ui::close_menu();
                                                        })
                                                        .child(
                                                            view()
                                                                .h(px(20.0))
                                                                .w(px(32.0))
                                                                .flex_row()
                                                                .rounded(px(4.0))
                                                                .overflow_hidden()
                                                                .border(px(1.0))
                                                                .border_color(move || {
                                                                    theme().border
                                                                })
                                                                .child(
                                                                    view()
                                                                        .flex_grow(1.0)
                                                                        .h_full()
                                                                        .bg(rgba(
                                                                            (definition.background
                                                                                << 8)
                                                                                | 0xff,
                                                                        )),
                                                                )
                                                                .child(
                                                                    view()
                                                                        .flex_grow(1.0)
                                                                        .h_full()
                                                                        .bg(rgba(
                                                                            (definition.card << 8)
                                                                                | 0xff,
                                                                        )),
                                                                )
                                                                .child(
                                                                    view().w(px(8.0)).h_full().bg(
                                                                        rgba(
                                                                            (definition.primary
                                                                                << 8)
                                                                                | 0xff,
                                                                        ),
                                                                    ),
                                                                ),
                                                        )
                                                        .child(
                                                            text(definition.label)
                                                                .flex_grow(1.0)
                                                                .number_of_lines(1)
                                                                .text_size(px(11.0))
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(move || {
                                                                    theme().foreground
                                                                }),
                                                        )
                                                        .into_element()
                                                }),
                                        ),
                                )
                                .into_element()
                        }),
                ),
            ),
        )
        .into_element()
}
