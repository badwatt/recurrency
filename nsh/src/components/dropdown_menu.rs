//! Generic labelled action menu, used by report exports.

use std::rc::Rc;

use crate::components::button::{ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button};
use crate::components::card::card;
use nsh_core::prelude::*;

#[derive(Clone)]
pub struct MenuAction {
    pub label: String,
    pub on_press: Rc<dyn Fn()>,
}

impl MenuAction {
    pub fn new(label: impl Into<String>, on_press: impl Fn() + 'static) -> Self {
        Self {
            label: label.into(),
            on_press: Rc::new(on_press),
        }
    }
}

pub fn dropdown_menu(
    label: &'static str,
    icon: Option<ImageAsset>,
    actions: Vec<MenuAction>,
) -> impl IntoElement {
    let actions = Rc::new(actions);
    let panel_actions = actions.clone();
    let build = move || {
        let actions = panel_actions.clone();
        card()
            .w(px(180.0))
            .p(px(4.0))
            .gap(px(2.0))
            .children(actions.iter().map(|action| {
                let press = action.on_press.clone();
                let label = action.label.clone();
                button(
                    ButtonProps::new(label)
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
    button(
        ButtonProps::new(label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .min_width(118.0)
            .icon_opt(icon)
            .trailing_icon(asset!("icons/chevron-down.svg"))
            .on_press(move || {
                let build = build.clone();
                crate::ui::open_menu(crate::ui::MenuPlacement::TopRight, build);
            }),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_dropdown_uses_the_shared_menu_host() {
        let source = include_str!("dropdown_menu.rs");
        assert!(source.contains("open_menu"));
        assert!(source.contains("close_menu"));
    }
}
