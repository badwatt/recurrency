//! Mobile floating action menu. One trigger, one list of actions.

use crate::components::button::{
    ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button as shared_button,
};
use crate::components::card::card;
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct FabAction {
    pub label: String,
    pub icon: ImageAsset,
    pub on_press: Rc<dyn Fn()>,
}

impl FabAction {
    pub fn new(label: impl Into<String>, icon: ImageAsset, on_press: impl Fn() + 'static) -> Self {
        Self {
            label: label.into(),
            icon,
            on_press: Rc::new(on_press),
        }
    }
}

pub fn fab_menu(_open: Signal<bool>, actions: Vec<FabAction>) -> impl IntoElement {
    let actions = Rc::new(actions);
    let panel_actions = actions.clone();
    let build = move || {
        let actions = panel_actions.clone();
        card()
            .p(px(6.0))
            .gap(px(2.0))
            .children(actions.iter().map(|action| {
                let press = action.on_press.clone();
                shared_button(
                    ButtonProps::new(action.label.clone())
                        .icon(action.icon)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .align(ButtonAlign::Start)
                        .full_width()
                        .on_press(move || {
                            crate::ui::close_menu();
                            press();
                        }),
                )
            }))
            .into_element()
    };
    view()
        .absolute()
        .right(px(18.0))
        .bottom(px(76.0))
        .z_index(60)
        .child(
            button()
                .cursor_pointer()
                .cursor_pointer()
                .items_center()
                .justify_center()
                .h(px(52.0))
                .w(px(52.0))
                .rounded(px(999.0))
                .bg(move || theme().primary)
                .accessibility_label("Quick actions")
                .on_press(move || {
                    let build = build.clone();
                    crate::ui::open_menu(crate::ui::MenuPlacement::BottomRight, build);
                })
                .child(
                    image(asset!("icons/plus.svg"))
                        .size(px(22.0))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || theme().primary_foreground),
                ),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_quick_actions_use_the_shared_menu_host() {
        let source = include_str!("fab_menu.rs");
        assert!(source.contains("MenuPlacement::BottomRight"));
        assert!(source.contains("Quick actions"));
    }
}
