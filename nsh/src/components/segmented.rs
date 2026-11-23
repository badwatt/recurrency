//! Segmented control for view switches and tab-like filters.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentOption {
    pub value: String,
    pub label: String,
}

impl SegmentOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub fn segmented(selected: Signal<String>, options: Vec<SegmentOption>) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .self_start()
        .gap(px(2.0))
        .p(px(3.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().muted.opacity(0.4))
        .children(options.into_iter().map(|option| {
            let value = option.value;
            let press_value = value.clone();
            let border_value = value.clone();
            let bg_value = value.clone();
            let text_value = value.clone();
            button()
                .cursor_pointer()
                .cursor_pointer()
                .h(px(28.0))
                .px(px(10.0))
                .items_center()
                .justify_center()
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || {
                    if selected.get() == border_value {
                        theme().primary
                    } else {
                        theme().border.opacity(0.0)
                    }
                })
                .bg(move || {
                    if selected.get() == bg_value {
                        theme().accent
                    } else {
                        theme().muted.opacity(0.0)
                    }
                })
                .on_press(move || selected.set(press_value.clone()))
                .child(
                    text(option.label)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if selected.get() == text_value {
                                theme().foreground
                            } else {
                                theme().muted_foreground
                            }
                        }),
                )
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_option_keeps_value_and_label() {
        let option = SegmentOption::new("grid", "Grid");
        assert_eq!(option.value, "grid");
        assert_eq!(option.label, "Grid");
    }
}
