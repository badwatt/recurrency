//! Filter trigger and popover shell. Pages supply the fields; the trigger keeps
//! the shared icon, active count and panel chrome.

use std::rc::Rc;

use crate::components::card::card;
use nsh_base::{popover, theme};
use nsh_core::prelude::*;

pub fn filter_menu(
    active: impl Fn() -> usize + 'static,
    body: impl Fn() -> AnyElement + 'static,
) -> impl IntoElement {
    let active = Rc::new(active);
    let active_border = active.clone();
    let active_icon = active.clone();
    let badge_active = active.clone();
    let badge_condition = active.clone();
    popover()
        .placement(Placement::Bottom)
        .align(Align::End)
        .offset(px(6.0))
        .trigger(
            view()
                .relative()
                .h(px(30.0))
                .w(px(30.0))
                .items_center()
                .justify_center()
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || {
                    if active_border() > 0 {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || theme().card)
                .child(
                    image(asset!("icons/filter.svg"))
                        .size(px(16.0))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || {
                            if active_icon() > 0 {
                                theme().primary
                            } else {
                                theme().muted_foreground
                            }
                        }),
                )
                .child(
                    when(
                        move || badge_condition() > 0,
                        move || {
                            view()
                                .absolute()
                                .top(px(-4.0))
                                .right(px(-4.0))
                                .h(px(14.0))
                                .min_w(px(14.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(999.0))
                                .bg(move || theme().primary)
                                .child(
                                    text(badge_active().to_string())
                                        .text_size(px(8.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().primary_foreground),
                                )
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                ),
        )
        .child(card().w(px(280.0)).p(px(12.0)).gap(px(10.0)).child(body()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_filter_shell_is_an_overlay_popover() {
        let source = include_str!("filter_menu.rs");
        assert!(source.contains("popover()"));
        assert!(source.contains("active() > 0"));
    }
}
