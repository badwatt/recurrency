//! Clickable dialog title used by every Money add/edit form.

use std::rc::Rc;

use nsh_base::{button, theme};
use nsh_core::prelude::*;

struct Choice {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    icon: ImageAsset,
}

fn choices() -> [Choice; 4] {
    [
        Choice {
            key: "subscription",
            label: "Subscription",
            description: "A recurring service like Netflix.",
            icon: asset!("icons/subscriptions.svg"),
        },
        Choice {
            key: "budget",
            label: "Budget",
            description: "A recurring expense you plan for.",
            icon: asset!("icons/wallet.svg"),
        },
        Choice {
            key: "transaction",
            label: "One-off payment",
            description: "A single charge.",
            icon: asset!("icons/transactions.svg"),
        },
        Choice {
            key: "debt",
            label: "Debt",
            description: "Money you owe.",
            icon: asset!("icons/debts.svg"),
        },
    ]
}

pub fn money_title(
    prefix: &'static str,
    current: impl Fn() -> String + 'static,
    on_switch: impl Fn(&'static str) + 'static,
) -> AnyElement {
    let on_switch = Rc::new(on_switch);
    let current = Rc::new(current);
    let panel = {
        let on_switch = on_switch.clone();
        move || {
            let rows = choices()
                .into_iter()
                .map(|choice| {
                    let on_switch = on_switch.clone();
                    button()
                        .cursor_pointer()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .gap(px(10.0))
                        .p(px(10.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .bg(move || theme().card)
                        .on_press(move || {
                            crate::ui::close_menu();
                            on_switch(choice.key);
                        })
                        .child(
                            view()
                                .h(px(32.0))
                                .w(px(32.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .bg(move || theme().primary.opacity(0.12))
                                .child(
                                    image(choice.icon)
                                        .size(px(16.0))
                                        .tint_color(move || theme().primary),
                                ),
                        )
                        .child(
                            view()
                                .flex_col()
                                .min_w(px(0.0))
                                .child(
                                    text(choice.label)
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text(choice.description)
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .into_element()
                })
                .collect::<Vec<_>>();
            crate::components::card::card()
                .w(px(300.0))
                .p(px(8.0))
                .gap(px(4.0))
                .children(rows)
                .into_element()
        }
    };
    let current_for_text = current.clone();
    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(
            text(prefix)
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px(px(6.0))
                .rounded(px(6.0))
                .cursor_pointer()
                .on_press(move |event: &PressEvent| {
                    let page = event.page_position;
                    let local = event.position;
                    let panel = panel.clone();
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: page.y.0 - local.y.0 + 28.0,
                            left: page.x.0 - local.x.0 - 6.0,
                        },
                        panel,
                    );
                })
                .child(
                    text(move || current_for_text())
                        .text_size(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary),
                )
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(14.0))
                        .tint_color(move || theme().primary),
                ),
        )
        .into_element()
}
