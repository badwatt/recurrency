//! Generic labelled action menu, used by report exports.

use std::rc::Rc;

use crate::components::button::{ButtonAlign, ButtonProps, ButtonSize, ButtonVariant, button};
use crate::components::card::card;
use nsh_base::popover;
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
    let panel_actions = actions.clone();
    popover()
        .placement(Placement::Bottom)
        .align(Align::End)
        .offset(px(6.0))
        .trigger(button(
            ButtonProps::new(label)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .icon_opt(icon),
        ))
        .child(
            card()
                .w(px(170.0))
                .p(px(4.0))
                .gap(px(2.0))
                .children(panel_actions.into_iter().map(|action| {
                    let press = action.on_press.clone();
                    button(
                        ButtonProps::new(action.label)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .align(ButtonAlign::Start)
                            .full_width()
                            .on_press(move || press()),
                    )
                })),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_dropdown_is_an_overlay_popover() {
        let source = include_str!("dropdown_menu.rs");
        assert!(source.contains("popover()"));
        assert!(source.contains("open.set(false)"));
    }
}
