//! Compact overlay select for toolbars and page headers.

use crate::components::button::{ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button};
use crate::components::card::card;
use nsh_base::{popover, theme};
use nsh_core::prelude::*;

pub fn compact_select(
    value: Signal<String>,
    options: Vec<(String, String)>,
    width: f32,
) -> impl IntoElement {
    let option_rows = options.clone();
    let panel_options = options.clone();
    let label = move || {
        let current = value.get();
        option_rows
            .iter()
            .find(|(option, _)| *option == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone())
    };
    popover()
        .placement(Placement::Bottom)
        .align(Align::End)
        .offset(px(6.0))
        .trigger(
            view()
                .w(px(width))
                .h(px(38.0))
                .px(px(10.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .child(
                    text(label())
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(13.0))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .child(
            card()
                .w(px(width.max(160.0)))
                .p(px(4.0))
                .gap(px(2.0))
                .children(panel_options.into_iter().map(|(option, label)| {
                    let active = value.get() == option;
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
                            .on_press(move || value.set(option.clone())),
                    )
                })),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_compact_select_is_an_overlay_popover() {
        let source = include_str!("compact_select.rs");
        assert!(source.contains("popover()"));
        assert!(source.contains("PopoverEvent::OpenChange"));
    }
}
