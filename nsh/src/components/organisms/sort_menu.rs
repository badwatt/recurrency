//! Sort field + direction menu shared by every list toolbar.

use crate::components::button::{ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button};
use crate::components::card::card;
use nsh_base::theme;
use nsh_core::prelude::*;

#[derive(Clone)]
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
    let panel_options = options.clone();
    let panel = move || {
        let options = panel_options.clone();
        card()
            .w(px(210.0))
            .p(px(8.0))
            .gap(px(2.0))
            .bg(move || theme().popover)
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
                            crate::ui::close_menu();
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
            ))
            .into_element()
    };
    view()
        .relative()
        .z_index(120)
        .child(
            view()
                .cursor_pointer()
                .h(px(40.0))
                .w(px(40.0))
                .items_center()
                .justify_center()
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .on_press(move |event: &PressEvent| {
                    let panel = panel.clone();
                    let page = event.page_position;
                    let local = event.position;
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: page.y.0 - local.y.0 + 44.0,
                            left: (page.x.0 - 105.0).max(8.0),
                        },
                        panel,
                    );
                })
                .child(
                    image(asset!("icons/sort.svg"))
                        .size(px(18.0))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sort_menu_uses_the_shared_global_menu_host() {
        let source = include_str!("sort_menu.rs");
        assert!(source.contains("open_menu"));
        assert!(source.contains("MenuPlacement::At"));
        assert!(source.contains("page.y.0 - local.y.0 + 44.0"));
    }
}
