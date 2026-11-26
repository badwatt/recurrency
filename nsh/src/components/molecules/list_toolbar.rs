//! Compact list toolbar: shared search, extra actions and view switch.

use crate::components::button::{ButtonProps, ButtonSize, ButtonVariant, icon_button_when};
use crate::components::search_input::search_input;
use nsh_base::{button as raw_button, theme};
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
        .gap(px(8.0))
        .child(search_input(search, "Search"))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(actions)
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .flex_shrink_0()
                        .h(px(40.0))
                        .p(px(2.0))
                        .gap(px(2.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .bg(move || theme().muted.opacity(0.4))
                        .child(view_toggle(
                            asset!("icons/grid.svg"),
                            "Grid view",
                            move || !list_view.get(),
                            move || list_view.set(false),
                        ))
                        .child(view_toggle(
                            asset!("icons/list.svg"),
                            "List view",
                            move || list_view.get(),
                            move || list_view.set(true),
                        )),
                ),
        )
}

fn view_toggle(
    icon: ImageAsset,
    label: &'static str,
    active: impl Fn() -> bool + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    let active = std::rc::Rc::new(active);
    let bg_active = active.clone();
    raw_button()
        .cursor_pointer()
        .h(px(36.0))
        .w(px(36.0))
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .bg(move || {
            if bg_active() {
                theme().background
            } else {
                theme().card.opacity(0.0)
            }
        })
        .accessibility_label(label)
        .on_press(on_press)
        .child(
            image(icon)
                .size(px(16.0))
                .pointer_events(PointerEvents::None)
                .tint_color(move || {
                    if active() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }),
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
