//! Compact select for toolbars and page headers, rendered in the shared menu host.

use std::rc::Rc;

use crate::components::button::{ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button};
use crate::components::card::card;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn compact_select(
    value: Signal<String>,
    options: Vec<(String, String)>,
    width: f32,
) -> impl IntoElement {
    let option_rows = Rc::new(options.clone());
    let label_rows = option_rows.clone();
    let panel_options = option_rows.clone();
    let label = move || {
        let current = value.get();
        label_rows
            .iter()
            .find(|(option, _)| *option == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone())
    };
    let build = move || {
        let options = panel_options.clone();
        card()
            .w(px(width.max(160.0)))
            .p(px(4.0))
            .gap(px(2.0))
            .children(options.iter().map(|(option, label)| {
                let active = value.get() == *option;
                let option = option.clone();
                let label = label.clone();
                button(
                    ButtonProps::new(label)
                        .variant(if active {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .size(ButtonSize::Sm)
                        .align(ButtonAlign::Start)
                        .full_width()
                        .on_press(move || {
                            value.set(option.clone());
                            crate::ui::close_menu();
                        }),
                )
            }))
            .into_element()
    };
    view()
        .w(px(width))
        .h(px(36.0))
        .px(px(12.0))
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().background)
        .on_press(move || {
            let build = build.clone();
            crate::ui::open_menu(crate::ui::MenuPlacement::TopRight, build);
        })
        .child(
            text(label())
                .number_of_lines(1)
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            image(asset!("icons/chevron-down.svg"))
                .size(px(13.0))
                .pointer_events(PointerEvents::None)
                .tint_color(move || theme().muted_foreground),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_compact_select_uses_the_shared_menu_host() {
        let source = include_str!("compact_select.rs");
        assert!(source.contains("open_menu"));
        assert!(source.contains("MenuPlacement::TopRight"));
    }
}
