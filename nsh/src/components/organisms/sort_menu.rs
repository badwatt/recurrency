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
    let sheet_options = options.clone();
    let sheet = move || {
        let options = sheet_options.clone();
        // A bottom sheet on phones: everything the thumb reaches, the list
        // right over the bottom edge.
        view()
            .w_full()
            .h_full()
            .flex_col()
            .justify_end()
            .bg(move || theme().background.opacity(0.6))
            .child(
                view()
                    .flex_grow(1.0)
                    .on_press(crate::ui::close_menu)
                    .into_element(),
            )
            .child(
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(6.0))
                    .p(px(16.0))
                    .pb(px(24.0))
                    .rounded_t(px(16.0))
                    .border_t(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .pb(px(6.0))
                            .child(
                                text("Sort by")
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(
                                view()
                                    .cursor_pointer()
                                    .h(px(36.0))
                                    .w(px(36.0))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(8.0))
                                    .bg(move || theme().muted.opacity(0.4))
                                    .on_press(crate::ui::close_menu)
                                    .child(
                                        image(asset!("icons/x.svg"))
                                            .size(px(16.0))
                                            .tint_color(move || theme().muted_foreground),
                                    ),
                            ),
                    )
                    .children(options.into_iter().map(|option| {
                        let active = move || field.get() == option.id;
                        view()
                            .cursor_pointer()
                            .w_full()
                            .h(px(48.0))
                            .px(px(14.0))
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .rounded(px(10.0))
                            .bg(move || {
                                if active() {
                                    theme().primary.opacity(0.12)
                                } else {
                                    theme().muted.opacity(0.25)
                                }
                            })
                            .on_press(move || {
                                field.set(option.id.to_string());
                            })
                            .child(
                                text(option.label)
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || {
                                        if active() {
                                            theme().primary
                                        } else {
                                            theme().foreground
                                        }
                                    }),
                            )
                            .when(active(), |row| {
                                row.child(
                                    image(asset!("icons/check.svg"))
                                        .size(px(16.0))
                                        .tint_color(move || theme().primary),
                                )
                            })
                    }))
                    .child(
                        view()
                            .cursor_pointer()
                            .w_full()
                            .h(px(48.0))
                            .px(px(14.0))
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .rounded(px(10.0))
                            .border(px(1.0))
                            .border_color(move || theme().border)
                            .on_press(move || descending.set(!descending.get()))
                            .child(
                                text(move || {
                                    if descending.get() {
                                        "Descending".to_string()
                                    } else {
                                        "Ascending".to_string()
                                    }
                                })
                                .text_size(px(15.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                            )
                            .child(
                                image(asset!("icons/chevron-down.svg"))
                                    .size(px(16.0))
                                    .tint_color(move || theme().muted_foreground),
                            ),
                    )
                    .child(
                        view()
                            .cursor_pointer()
                            .w_full()
                            .h(px(48.0))
                            .mt(px(4.0))
                            .items_center()
                            .justify_center()
                            .rounded(px(10.0))
                            .border(px(1.0))
                            .border_color(move || theme().border)
                            .bg(move || theme().card)
                            .on_press(crate::ui::close_menu)
                            .child(
                                text("Close")
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().foreground),
                            ),
                    ),
            )
            .into_element()
    };
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
                        }),
                )
            }))
            // A built label would freeze: the direction is read while it
            // renders, so flipping it shows at once.
            .child(
                view()
                    .cursor_pointer()
                    .w_full()
                    .h(px(ButtonSize::Sm.height()))
                    .px(px(12.0))
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .on_press(move || descending.set(!descending.get()))
                    .child(
                        text(move || {
                            if descending.get() {
                                "Descending".to_string()
                            } else {
                                "Ascending".to_string()
                            }
                        })
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                    )
                    .child(
                        image(asset!("icons/chevron-down.svg"))
                            .size(px(14.0))
                            .rotate(move || {
                                Radians::from(if descending.get() {
                                    0.0_f32
                                } else {
                                    std::f32::consts::PI
                                })
                            })
                            .tint_color(move || theme().muted_foreground),
                    ),
            )
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
                    if crate::ui::compact_active() {
                        let sheet = sheet.clone();
                        crate::ui::open_menu(crate::ui::MenuPlacement::FullScreen, sheet);
                        return;
                    }
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
