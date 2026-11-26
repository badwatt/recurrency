//! Filter trigger and panel. Pages supply the fields; the panel renders in the
//! app-wide menu host so it is never clipped by list content.

use std::rc::Rc;

use crate::components::card::card;
use nsh_base::theme;
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
    let body = Rc::new(body);
    let panel_body = body.clone();
    let panel = move || {
        card()
            .w(px(300.0))
            .p(px(12.0))
            .gap(px(10.0))
            .bg(move || theme().popover)
            .child(panel_body())
            .into_element()
    };
    view()
        .relative()
        .child(
            view()
                .cursor_pointer()
                .h(px(40.0))
                .w(px(40.0))
                .items_center()
                .justify_center()
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || {
                    if active_border() > 0 {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || theme().card)
                .on_press(move |event: &PressEvent| {
                    let panel = panel.clone();
                    let page = event.page_position;
                    let local = event.position;
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: page.y.0 - local.y.0 + 44.0,
                            left: (page.x.0 - 150.0).max(8.0),
                        },
                        panel,
                    );
                })
                .child(
                    image(asset!("icons/filter.svg"))
                        .size(px(18.0))
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
        .into_element()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_filter_menu_uses_the_shared_menu_host() {
        let source = include_str!("filter_menu.rs");
        assert!(source.contains("open_menu"));
        assert!(source.contains("MenuPlacement::At"));
        assert!(source.contains("page.y.0 - local.y.0 + 44.0"));
    }
}
