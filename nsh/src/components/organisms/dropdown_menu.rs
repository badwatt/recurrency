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
                            press();
                            crate::ui::close_menu();
                        }),
                )
            }))
            .into_element()
    };
    view()
        .cursor_pointer()
        .h(px(36.0))
        .w(px(121.0))
        .px(px(12.0))
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color(move || nsh_base::theme().border)
        .bg(move || nsh_base::theme().card)
        .on_press(move |event: &PressEvent| {
            let build = build.clone();
            let page = event.page_position;
            crate::ui::open_menu(
                crate::ui::MenuPlacement::At {
                    top: page.y.0 + 28.0,
                    left: (page.x.0 - 120.0).max(8.0),
                },
                build,
            );
        })
        .child(
            image(icon.unwrap_or_else(|| asset!("icons/download.svg")))
                .size(px(15.0))
                .tint_color(move || nsh_base::theme().muted_foreground),
        )
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || nsh_base::theme().foreground),
        )
        .child(
            image(asset!("icons/chevron-down.svg"))
                .size(px(13.0))
                .tint_color(move || nsh_base::theme().muted_foreground),
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
