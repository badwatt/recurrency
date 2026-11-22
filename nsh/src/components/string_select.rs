//! Searchable single-choice menu for string-valued catalogues (currencies).

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub struct StringSelectProps {
    pub label: String,
    pub selected: Signal<String>,
    pub options: Vec<(String, String)>,
}

impl StringSelectProps {
    pub fn new(label: impl Into<String>, selected: Signal<String>, options: Vec<(String, String)>) -> Self {
        Self { label: label.into(), selected, options }
    }
}

pub fn string_select(props: StringSelectProps) -> impl IntoElement {
    let StringSelectProps { label, selected, options } = props;
    let open = signal(false);
    let query = signal(String::new());
    let option_rows = options.clone();
    let selected_label = move || {
        let current = selected.get();
        options
            .iter()
            .find(|(value, _)| *value == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone())
    };
    let filtered = move || {
        let q = query.get().trim().to_lowercase();
        option_rows
            .iter()
            .filter(|(value, label)| q.is_empty() || value.to_lowercase().contains(&q) || label.to_lowercase().contains(&q))
            .cloned()
            .collect::<Vec<_>>()
    };
    view()
        .flex_grow(1.0)
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(text(label).text_size(px(12.0)).font_weight(FontWeight::MEDIUM).text_color(move || theme().muted_foreground))
        .child(
            button()
                .w_full()
                .h(px(38.0))
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
                .child(text(selected_label()).number_of_lines(1).text_size(px(13.0)).text_color(move || theme().foreground))
                .child(image(asset!("icons/chevron-down.svg")).size(px(13.0)).tint_color(move || theme().muted_foreground)),
        )
        .child(
            when(open, move || {
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(4.0))
                    .p(px(4.0))
                    .rounded(px(7.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().popover)
                    .child(
                        input()
                            .value(query)
                            .on_change(move |value: &str| query.set(value.to_string()))
                            .placeholder("Search currencies…")
                            .auto_fill(AutoFill::Off)
                            .h(px(30.0))
                            .w_full()
                            .px(px(8.0))
                            .rounded(px(5.0))
                            .border(px(1.0))
                            .border_color(move || theme().input)
                            .bg(move || theme().background)
                            .text_size(px(11.0))
                            .text_color(move || theme().foreground),
                    )
                    .child(
                        scroll()
                            .max_h(px(220.0))
                            .w_full()
                            .children(filtered().into_iter().map(|(value, label)| {
                                let press_value = value.clone();
                                let active = selected.get() == value;
                                button()
                                    .w_full()
                                    .h(px(28.0))
                                    .px(px(7.0))
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .rounded(px(5.0))
                                    .bg(move || if active { theme().accent } else { theme().popover })
                                    .on_press(move || {
                                        selected.set(press_value.clone());
                                        open.set(false);
                                        query.set(String::new());
                                    })
                                    .child(text(label).number_of_lines(1).text_size(px(11.0)).text_color(move || theme().foreground))
                                    .when(active, |row| row.child(image(asset!("icons/check.svg")).size(px(12.0)).tint_color(move || theme().primary)))
                                    .into_element()
                            })),
                    )
                    .into_element()
            })
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}
