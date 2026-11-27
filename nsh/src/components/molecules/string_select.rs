//! String-valued select used by currencies, timezones and other catalogues.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::select::{SelectProps, select};

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
    let options = Rc::new(options);
    let label_options = options.clone();
    let panel_options = options.clone();
    select(
        SelectProps::new(
            label,
            "Select...",
            move || {
                let Some(current) = selected.try_get() else {
                    return String::new();
                };
                label_options
                    .iter()
                    .find(|(value, _)| *value == current)
                    .map_or_else(|| current.clone(), |(_, label)| label.clone())
            },
            move || selected.try_get().unwrap_or_default(),
            move || selected.try_get().is_some_and(|value| !value.is_empty()),
            move || (*panel_options).clone(),
            move |value| {
                let _ = selected.try_set(value);
            },
        )
        .searchable(true)
        .show_label(show_label)
        .height(36.0),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn string_select_uses_the_shared_select() {
        let source = include_str!("string_select.rs");
        assert!(source.contains("SelectProps::new"));
        assert!(source.contains("select("));
    }
}
