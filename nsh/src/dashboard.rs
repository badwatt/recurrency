//! Dashboard shell: the visual frame is ported first, data wiring follows.

use crate::ui;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn dashboard() -> AnyElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_end()
                .justify_between()
                .child(ui::page_header(
                    "This month's plan",
                    "What to transfer into each account, and what is still left to pay.",
                ))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .child(action("Add subscription", true))
                        .child(action("Add transaction", false))
                        .child(action("Add debt", false)),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(0.0))
                .child(
                    ui::surface()
                        .flex_grow(3.0)
                        .p(px(32.0))
                        .border_r(px(1.0))
                        .child(
                            text("PLAN FOR NOVEMBER 2026")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground)
                                .mb(px(18.0)),
                        )
                        .child(
                            text("Transfer this month")
                                .text_size(px(15.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text("€0.00")
                                .text_size(px(56.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    ui::surface()
                        .flex_grow(2.0)
                        .p(px(32.0))
                        .child(
                            text("PENDING FROM TODAY")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground)
                                .mb(px(18.0)),
                        )
                        .child(
                            text("€0.00")
                                .text_size(px(32.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(16.0))
                .child(
                    ui::surface()
                        .flex_grow(3.0)
                        .p(px(24.0))
                        .child(
                            text("UPCOMING PAYMENTS")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    ui::surface()
                        .flex_grow(2.0)
                        .p(px(24.0))
                        .child(
                            text("TRANSFER BY ACCOUNT")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .into_element()
}

fn action(label: &'static str, primary: bool) -> impl IntoElement {
    button()
        .h(px(42.0))
        .px(px(16.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || if primary { theme().primary } else { theme().card })
        .active(|s| s.opacity(0.8))
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || if primary {
                    theme().primary_foreground
                } else {
                    theme().foreground
                }),
        )
}
