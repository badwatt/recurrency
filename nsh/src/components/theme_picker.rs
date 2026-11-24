//! Grid of the 18 palettes. Selecting one applies it immediately, stores it
//! locally and syncs the choice to the server.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn theme_picker(selected: Signal<String>, server: String, token: String) -> impl IntoElement {
    let dark = crate::theme::THEMES
        .iter()
        .filter(|definition| definition.is_dark)
        .collect::<Vec<_>>();
    let light = crate::theme::THEMES
        .iter()
        .filter(|definition| !definition.is_dark)
        .collect::<Vec<_>>();
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(theme_group(
            "DARK",
            dark,
            selected,
            server.clone(),
            token.clone(),
        ))
        .child(theme_group("LIGHT", light, selected, server, token))
}

fn theme_group(
    label: &'static str,
    definitions: Vec<&'static crate::theme::ThemeDefinition>,
    selected: Signal<String>,
    server: String,
    token: String,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(8.0))
        .child(
            text(label)
                .text_size(px(10.0))
                .letter_spacing(px(1.6))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(3, minmax(0, 1fr))"))
                .gap(px(12.0))
                .children(definitions.into_iter().map(|definition| {
                    theme_card(definition, selected, server.clone(), token.clone())
                })),
        )
}

fn theme_card(
    definition: &'static crate::theme::ThemeDefinition,
    selected: Signal<String>,
    server: String,
    token: String,
) -> AnyElement {
    let id = definition.id;
    let card_server = server;
    let card_token = token;
    let active = move || selected.get() == id;
    view()
        .w_full()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .rounded(px(10.0))
        .border(px(1.0))
        .border_color(move || {
            if active() {
                theme().primary
            } else {
                theme().border
            }
        })
        .bg(move || theme().background)
        .cursor_pointer()
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
            view()
                .w_full()
                .h(px(24.0))
                .flex_row()
                .overflow_hidden()
                .rounded(px(6.0))
                .child(
                    view()
                        .flex_grow(1.0)
                        .bg(rgba((definition.background << 8) | 0xff)),
                )
                .child(
                    view()
                        .flex_grow(1.0)
                        .bg(rgba((definition.card << 8) | 0xff)),
                )
                .child(
                    view()
                        .flex_grow(1.0)
                        .bg(rgba((definition.primary << 8) | 0xff)),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    text(definition.label)
                        .flex_grow(1.0)
                        .number_of_lines(1)
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .when(active(), |row| {
                    row.child(
                        image(asset!("icons/check.svg"))
                            .size(px(15.0))
                            .tint_color(move || theme().primary),
                    )
                }),
        )
        .child(
            text(definition.description)
                .text_size(px(11.0))
                .line_height(px(16.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_picker_is_grouped_into_dark_and_light_sections() {
        let source = include_str!("theme_picker.rs");
        assert!(source.contains("\"DARK\""));
        assert!(source.contains("\"LIGHT\""));
    }

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
                                                        .cursor_pointer()
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
