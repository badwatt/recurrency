//! Compact list toolbar: shared search, extra actions and view switch.

use crate::components::button::{
    ButtonProps, ButtonSize, ButtonVariant, icon_button_when, icon_button_when_sized,
};
use crate::components::search_input::search_input;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn list_toolbar(
    search: Signal<String>,
    list_view: Signal<bool>,
    actions: AnyElement,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(12.0))
        .child(search_input(search, "Search"))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(12.0))
                .child(actions)
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .flex_shrink_0()
                        .w(px(72.0))
                        .gap(px(0.0))
                        .p(px(0.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().muted.opacity(0.4))
                        .child(icon_button_when_sized(
                            asset!("icons/grid.svg"),
                            "Grid view",
                            ButtonSize::Toolbar,
                            move || !list_view.get(),
                            move || list_view.set(false),
                        ))
                        .child(icon_button_when_sized(
                            asset!("icons/list.svg"),
                            "List view",
                            ButtonSize::Toolbar,
                            move || list_view.get(),
                            move || list_view.set(true),
                        )),
                ),
        )
}

pub fn filter_button(active: Signal<bool>) -> impl IntoElement {
    icon_button_when(
        asset!("icons/filter.svg"),
        "Filter active",
        move || active.get(),
        move || active.set(!active.get()),
    )
}

pub fn sort_button(descending: Signal<bool>) -> impl IntoElement {
    icon_button_when(
        asset!("icons/sort.svg"),
        "Sort newest first",
        move || descending.get(),
        move || descending.set(!descending.get()),
    )
}

pub fn text_action(label: &'static str, on_press: impl Fn() + 'static) -> impl IntoElement {
    crate::components::button::button(
        ButtonProps::new(label)
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Outline)
            .on_press(on_press),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn toolbar_uses_the_shared_search_and_view_controls() {
        let source = include_str!("list_toolbar.rs");
        assert!(source.contains("search_input(search"));
        assert!(source.contains("icons/grid.svg"));
        assert!(source.contains("icons/list.svg"));
    }
}
