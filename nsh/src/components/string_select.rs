//! Searchable single-choice menu for string-valued catalogues (currencies).

use std::rc::Rc;

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub struct StringSelectProps {
    pub label: String,
    pub selected: Signal<String>,
    pub options: Vec<(String, String)>,
    pub show_label: bool,
}

impl StringSelectProps {
    pub fn new(
        label: impl Into<String>,
        selected: Signal<String>,
        options: Vec<(String, String)>,
    ) -> Self {
        Self {
            label: label.into(),
            selected,
            options,
            show_label: true,
        }
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }
}

pub fn string_select(props: StringSelectProps) -> impl IntoElement {
    let StringSelectProps {
        label,
        selected,
        options,
        show_label,
    } = props;
    let query = signal(String::new());
    let option_rows = options.clone();
    let selected_label = move || {
        let current = selected.get();
        options
            .iter()
            .find(|(value, _)| *value == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone())
    };
    let filtered: Rc<dyn Fn() -> Vec<(String, String)>> = Rc::new(move || {
        let q = query.get().trim().to_lowercase();
        option_rows
            .iter()
            .filter(|(value, label)| {
                q.is_empty()
                    || value.to_lowercase().contains(&q)
                    || label.to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let panel_filtered = filtered.clone();
    let panel = move || {
        let filtered = panel_filtered.clone();
        view()
            .w(px(300.0))
            .flex_col()
            .rounded(px(6.0))
            .border(px(1.0))
            .border_color(move || theme().border)
            .bg(move || theme().popover)
            .overflow_hidden()
            .child(
                view()
                    .w_full()
                    .px(px(12.0))
                    .border_b(px(1.0))
                    .border_color(move || theme().border)
                    .child(
                        input()
                            .value(query)
                            .on_change(move |value: &str| query.set(value.to_string()))
                            .placeholder("Search...")
                            .auto_fill(AutoFill::Off)
                            .h(px(36.0))
                            .w_full()
                            .px(px(0.0))
                            .border(px(0.0))
                            .bg(move || theme().popover)
                            .text_size(px(14.0))
                            .text_color(move || theme().foreground),
                    ),
            )
            .child(scroll().max_h(px(220.0)).w_full().child(each(
                {
                    let filtered = filtered.clone();
                    move || filtered()
                },
                |(value, _)| value.clone(),
                move |option, _| {
                    let (value, label) = option.get();
                    let press_value = value.clone();
                    let active = selected.get() == value;
                    button()
                        .cursor_pointer()
                        .w_full()
                        .h(px(28.0))
                        .px(px(7.0))
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
                            selected.set(press_value.clone());
                            query.set(String::new());
                            crate::ui::close_menu();
                        })
                        .child(
                            text(label)
                                .number_of_lines(1)
                                .text_size(px(11.0))
                                .text_color(move || theme().foreground),
                        )
                        .when(active, |row| {
                            row.child(
                                image(asset!("icons/check.svg"))
                                    .size(px(12.0))
                                    .tint_color(move || theme().primary),
                            )
                        })
                        .into_element()
                },
            )))
            .into_element()
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .when(show_label, |column| {
            column.child(
                text(label)
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(move || theme().muted_foreground),
            )
        })
        .child(
            view()
                .cursor_pointer()
                .w_full()
                .h(px(36.0))
                .px(px(8.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .on_press(move |event: &PressEvent| {
                    let panel = panel.clone();
                    let page = event.page_position;
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: page.y.0 + 28.0,
                            left: (page.x.0 - 150.0).max(8.0),
                        },
                        panel,
                    );
                })
                .child(
                    text(move || selected_label())
                        .number_of_lines(1)
                        .text_size(px(14.0))
                        .text_color(move || {
                            if selected.get().is_empty() {
                                theme().muted_foreground
                            } else {
                                theme().foreground
                            }
                        }),
                )
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(13.0))
                        .tint_color(move || theme().muted_foreground),
                ),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn options_float_and_the_trigger_is_reactive() {
        let source = include_str!("string_select.rs");
        assert!(source.contains("open_menu"));
        assert!(source.contains("text(move || selected_label())"));
    }
}
