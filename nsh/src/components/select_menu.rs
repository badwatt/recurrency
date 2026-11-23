//! Compact single-choice menu for account and category fields. The trigger
//! and the option rows are shared so every picker has the same height,
//! border and selected state.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: Uuid,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: Uuid, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

pub struct SelectMenuProps {
    pub label: String,
    pub placeholder: String,
    pub selected: Signal<Option<Uuid>>,
    pub options: Rc<dyn Fn() -> Vec<SelectOption>>,
}

impl SelectMenuProps {
    pub fn new(
        label: impl Into<String>,
        placeholder: impl Into<String>,
        selected: Signal<Option<Uuid>>,
        options: impl Fn() -> Vec<SelectOption> + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            placeholder: placeholder.into(),
            selected,
            options: Rc::new(options),
        }
    }
}

pub fn select_menu(props: SelectMenuProps) -> impl IntoElement {
    let SelectMenuProps {
        label,
        placeholder,
        selected,
        options,
    } = props;
    let open = signal(false);
    let option_rows = options.clone();
    let selected_label = move || {
        let current = selected.get();
        options()
            .into_iter()
            .find(|option| Some(option.value) == current)
            .map_or_else(|| placeholder.clone(), |option| option.label)
    };
    view()
        .flex_grow(1.0)
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(
            text(label)
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            button()
                .cursor_pointer()
                .cursor_pointer()
                .w_full()
                .h(px(34.0))
                .px(px(10.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .on_press(move || open.set(!open.get()))
                .child(
                    text(selected_label())
                        .number_of_lines(1)
                        .text_size(px(13.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(13.0))
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(open, move || {
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(1.0))
                    .p(px(3.0))
                    .rounded(px(7.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().popover)
                    .children(option_rows().into_iter().map(|option| {
                        let id = option.value;
                        let active = selected.get() == Some(id);
                        button()
                            .cursor_pointer()
                            .w_full()
                            .h(px(30.0))
                            .px(px(8.0))
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .rounded(px(5.0))
                            .bg(move || {
                                if active {
                                    theme().accent
                                } else {
                                    theme().popover
                                }
                            })
                            .on_press(move || {
                                selected.set(Some(id));
                                open.set(false);
                            })
                            .child(
                                text(option.label)
                                    .number_of_lines(1)
                                    .text_size(px(12.0))
                                    .text_color(move || theme().foreground),
                            )
                            .when(active, |row| {
                                row.child(
                                    image(asset!("icons/check.svg"))
                                        .size(px(13.0))
                                        .tint_color(move || theme().primary),
                                )
                            })
                    }))
                    .into_element()
            })
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_option_keeps_value_and_label() {
        let id = Uuid::nil();
        let option = SelectOption::new(id, "General");
        assert_eq!(option.value, id);
        assert_eq!(option.label, "General");
    }
}
