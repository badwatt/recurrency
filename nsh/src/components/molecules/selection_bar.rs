//! Inline bulk-selection bar shared by the money lists.

use crate::components::button::{ButtonProps, ButtonSize, ButtonVariant};
use crate::ui;
use nsh_base::theme;
use nsh_core::prelude::*;
use uuid::Uuid;

pub fn selection_bar(
    count: impl Fn() -> usize + 'static,
    select_all: impl Fn() + 'static,
    clear: impl Fn() + 'static,
    actions: impl Fn() + Clone + 'static,
    cancel: impl Fn() + Clone + 'static,
) -> impl IntoElement {
    let actions_button = actions.clone();
    let cancel_button = cancel.clone();
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(px(10.0))
        .p(px(10.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().primary.opacity(0.35))
        .bg(move || theme().primary.opacity(0.08))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(
                    text(move || format!("{} selected", count()))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(ui::action_button("Select all", false, select_all))
                .child(ui::action_button("Clear", false, clear)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(crate::components::button::button(
                    ButtonProps::new("Bulk actions")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Xs)
                        .on_press(move || actions_button()),
                ))
                .child(crate::components::button::button(
                    ButtonProps::new("Done")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Xs)
                        .on_press(move || cancel_button()),
                )),
        )
}

/// Add or remove one id, preserving insertion order.
pub fn toggle(values: &mut Vec<Uuid>, id: Uuid) {
    if let Some(index) = values.iter().position(|value| *value == id) {
        values.remove(index);
    } else {
        values.push(id);
    }
}

/// Add every id that is not selected yet.
pub fn add_all(values: &mut Vec<Uuid>, ids: impl IntoIterator<Item = Uuid>) {
    for id in ids {
        if !values.contains(&id) {
            values.push(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    #[test]
    fn toggle_adds_then_removes_the_same_id() {
        let mut values = Vec::new();
        toggle(&mut values, id(1));
        assert_eq!(values, vec![id(1)]);
        toggle(&mut values, id(1));
        assert!(values.is_empty());
    }

    #[test]
    fn add_all_keeps_order_and_skips_duplicates() {
        let mut values = vec![id(2)];
        add_all(&mut values, [id(1), id(2), id(3)]);
        assert_eq!(values, vec![id(2), id(1), id(3)]);
    }
}
