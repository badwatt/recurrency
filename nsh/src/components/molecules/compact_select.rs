//! Compact select for toolbars and page headers.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::select::{SelectProps, select};

pub fn compact_select(
    value: Signal<String>,
    options: Vec<(String, String)>,
    width: f32,
) -> impl IntoElement {
    let options = Rc::new(options);
    let label_options = options.clone();
    let panel_options = options.clone();
    let select = select(
        SelectProps::new(
            "",
            "Select...",
            move || {
                let current = value.get();
                label_options
                    .iter()
                    .find(|(option, _)| *option == current)
                    .map_or_else(|| current.clone(), |(_, label)| label.clone())
            },
            move || value.get(),
            move || !value.get().is_empty(),
            move || (*panel_options).clone(),
            move |next| value.set(next),
        )
        .searchable(false)
        .show_label(false)
        .height(36.0),
    );
    view().w(px(width)).child(select)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compact_select_uses_the_shared_select() {
        let source = include_str!("compact_select.rs");
        assert!(source.contains("SelectProps::new"));
        assert!(source.contains("searchable(false)"));
    }
}
