//! Shared month-day selector for subscriptions and automatic debt payments.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn charge_days(label: &'static str, days: Signal<Vec<u8>>) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text(label)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            button()
                                .cursor_pointer()
                                .h(px(24.0))
                                .px(px(6.0))
                                .rounded(px(4.0))
                                .on_press(move || days.set((1u8..=31).collect()))
                                .child(
                                    text("Select all")
                                        .text_size(px(11.0))
                                        .text_color(move || theme().primary),
                                ),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .h(px(24.0))
                                .px(px(6.0))
                                .rounded(px(4.0))
                                .on_press(move || days.set(Vec::new()))
                                .child(
                                    text("Clear")
                                        .text_size(px(11.0))
                                        .text_color(move || theme().primary),
                                ),
                        ),
                ),
        )
        .child(day_grid(days))
        .child(
            text("Days 29-31 use 'last day' for short months. Each selected day is a separate charge.")
                .text_size(px(10.0))
                .mt(px(9.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn day_grid(days: Signal<Vec<u8>>) -> impl IntoElement {
    view()
        .w_full()
        .grid()
        .grid_template_columns(SharedString::from("repeat(11, minmax(0, 1fr))"))
        .gap(px(4.0))
        .mt(px(-4.0))
        .children((1u8..=31).map(|day| {
            let active = move || days.get().contains(&day);
            button()
                .cursor_pointer()
                .items_center()
                .justify_center()
                .h(px(32.0))
                .w_full()
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || {
                    if active() {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || {
                    if active() {
                        theme().primary
                    } else {
                        theme().card
                    }
                })
                .on_press(move || {
                    let mut values = days.get();
                    if let Some(index) = values.iter().position(|value| *value == day) {
                        values.remove(index);
                    } else {
                        values.push(day);
                        values.sort_unstable();
                    }
                    days.set(values);
                })
                .child(
                    text(day.to_string())
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if active() {
                                theme().primary_foreground
                            } else {
                                theme().foreground
                            }
                        }),
                )
        }))
        .child({
            let active = move || days.get().contains(&0);
            button()
                .cursor_pointer()
                .items_center()
                .justify_center()
                .h(px(32.0))
                .w_full()
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || {
                    if active() {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || {
                    if active() {
                        theme().primary
                    } else {
                        theme().card
                    }
                })
                .on_press(move || {
                    let mut selected = days.get();
                    if selected.contains(&0) {
                        selected.retain(|day| *day != 0);
                    } else {
                        selected.retain(|day| *day < 29);
                        selected.push(0);
                    }
                    days.set(selected);
                })
                .child(
                    text("Last day")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if active() {
                                theme().primary_foreground
                            } else {
                                theme().foreground
                            }
                        }),
                )
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_shared_selector_keeps_month_days_and_last_day_together() {
        let source = include_str!("charge_days.rs");
        assert!(source.contains("Select all"));
        assert!(source.contains("Last day"));
        assert!(source.contains("repeat(11, minmax(0, 1fr))"));
    }
}
