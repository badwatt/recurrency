//! Filter trigger and popover shell. Pages supply the fields; the trigger keeps
//! the shared icon, active count and panel chrome.

use crate::components::button::icon_button;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn filter_menu(active: usize, body: AnyElement) -> impl IntoElement {
    let open = signal(false);
    view()
        .relative()
        .child(icon_button(
            asset!("icons/filter.svg"),
            "Filters",
            active > 0,
            move || open.set(!open.get()),
        ))
        .when(active > 0, |badge| {
            badge.child(
                view()
                    .absolute()
                    .top(px(1.0))
                    .right(px(1.0))
                    .h(px(13.0))
                    .min_w(px(13.0))
                    .items_center()
                    .justify_center()
                    .rounded(px(999.0))
                    .bg(move || theme().primary)
                    .child(
                        text(active.to_string())
                            .text_size(px(8.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(move || theme().primary_foreground),
                    ),
            )
        })
        .when(open.get(), move |panel| {
            panel.child(
                view()
                    .absolute()
                    .top(px(38.0))
                    .right(px(0.0))
                    .z_index(80)
                    .w(px(280.0))
                    .p(px(12.0))
                    .rounded(px(9.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .gap(px(10.0))
                    .child(body),
            )
        })
}
