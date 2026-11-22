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
