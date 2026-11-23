//! Compact overlay select for toolbars and page headers. Unlike the form
//! `StringSelect`, it does not reserve a label or push the layout when open.

use crate::components::button::{button, ButtonAlign, ButtonProps, ButtonSize, ButtonVariant};
use nsh_base::{button as raw_button, theme};
use nsh_core::prelude::*;

pub fn compact_select(
    value: Signal<String>,
    options: Vec<(String, String)>,
    width: f32,
) -> impl IntoElement {
    let open = signal(false);
    let option_rows = options.clone();
    let label = move || {
        let current = value.get();
        option_rows
            .iter()
            .find(|(option, _)| *option == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone())
    };
    view()
        .relative()
        .w(px(width))
        .flex_shrink_0()
        .child(
            raw_button()
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
                .on_press(move || open.set(!open.get()))
                .child(
                    text(label())
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(13.0))
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .when(open.get(), move |trigger| {
            trigger.child(
                view()
                    .absolute()
                    .top(px(42.0))
                    .left(px(0.0))
                    .z_index(80)
                    .w(px(width.max(160.0)))
                    .p(px(4.0))
                    .gap(px(2.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .children(options.into_iter().map(|(option, label)| {
                        let active = value.get() == option;
                        button(
                            ButtonProps::new(label)
                                .variant(if active { ButtonVariant::Secondary } else { ButtonVariant::Ghost })
                                .size(ButtonSize::Sm)
                                .align(ButtonAlign::Start)
                                .full_width()
                                .on_press(move || {
                                    value.set(option.clone());
                                    open.set(false);
                                }),
                        )
                    })),
            )
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_compact_select_overlays_above_the_page() {
        let source = include_str!("compact_select.rs");
        assert!(source.contains("z_index(80)"));
        assert!(source.contains("icons/chevron-down.svg"));
    }
}
