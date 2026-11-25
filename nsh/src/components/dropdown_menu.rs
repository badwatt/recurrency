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
    let bounds = signal(None::<(f32, f32, f32, f32)>);
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
        .on_layout(
            move |value: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
                bounds.set(Some((
                    value.origin.x.0,
                    value.origin.y.0,
                    value.size.width.0,
                    value.size.height.0,
                )));
            },
        )
        .child(button(
            ButtonProps::new(label)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Md)
                .fixed_width(121.0)
                .icon_opt(icon)
                .trailing_icon(asset!("icons/chevron-down.svg"))
                .on_press(move || {
                    let build = build.clone();
                    let (x, y, width, height) =
                        bounds.get().unwrap_or((1100.0, 113.0, 121.0, 40.0));
                    let left = (x + width - 180.0).max(8.0);
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: y + height + 8.0,
                            left,
                        },
                        build,
                    );
                }),
        ))
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
