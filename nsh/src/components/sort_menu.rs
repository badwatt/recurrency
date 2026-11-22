//! Sort field + direction menu shared by every list toolbar.

use crate::components::button::{
    ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button, icon_button,
};
use nsh_base::theme;
use nsh_core::prelude::*;

pub struct SortOption {
    pub id: &'static str,
    pub label: &'static str,
}

impl SortOption {
    pub const fn new(id: &'static str, label: &'static str) -> Self {
        Self { id, label }
    }
}

pub fn sort_menu(
    field: Signal<String>,
    descending: Signal<bool>,
    options: Vec<SortOption>,
) -> impl IntoElement {
    let open = signal(false);
    view()
        .relative()
        .child(icon_button(
            asset!("icons/sort.svg"),
            "Sort",
            open.get(),
            move || open.set(!open.get()),
        ))
        .when(open.get(), move |panel| {
            panel.child(
                view()
                    .absolute()
                    .top(px(38.0))
                    .right(px(0.0))
                    .z_index(80)
                    .w(px(210.0))
                    .p(px(8.0))
                    .rounded(px(9.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .gap(px(2.0))
                    .child(
                        text("Sort by")
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().muted_foreground),
                    )
                    .children(options.into_iter().map(|option| {
                        let active = field.get() == option.id;
                        button(
                            ButtonProps::new(option.label)
                                .variant(if active {
                                    ButtonVariant::Secondary
                                } else {
                                    ButtonVariant::Ghost
                                })
                                .size(ButtonSize::Sm)
                                .align(ButtonAlign::Start)
                                .full_width()
                                .on_press(move || {
                                    field.set(option.id.to_string());
                                    open.set(false);
                                }),
                        )
                    }))
                    .child(button(
                        ButtonProps::new(if descending.get() {
                            "Descending"
                        } else {
                            "Ascending"
                        })
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .align(ButtonAlign::Between)
                        .full_width()
                        .on_press(move || descending.set(!descending.get())),
                    )),
            )
        })
}
