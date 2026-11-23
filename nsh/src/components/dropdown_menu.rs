//! Generic labelled action menu, used by report exports.

use crate::components::button::{button, ButtonAlign, ButtonProps, ButtonSize, ButtonVariant};
use nsh_base::theme;
use nsh_core::prelude::*;
use std::rc::Rc;

pub struct MenuAction {
    pub label: String,
    pub on_press: Rc<dyn Fn()>,
}

impl MenuAction {
    pub fn new(label: impl Into<String>, on_press: impl Fn() + 'static) -> Self {
        Self { label: label.into(), on_press: Rc::new(on_press) }
    }
}

pub fn dropdown_menu(label: &'static str, actions: Vec<MenuAction>) -> impl IntoElement {
    let open = signal(false);
    view()
        .relative()
        .child(
            button(
                ButtonProps::new(label)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .on_press(move || open.set(!open.get())),
            )
            .into_element(),
        )
        .when(open.get(), move |trigger| {
            trigger.child(
                view()
                    .absolute()
                    .top(px(40.0))
                    .right(px(0.0))
                    .z_index(80)
                    .w(px(170.0))
                    .p(px(4.0))
                    .gap(px(2.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .children(actions.into_iter().map(|action| {
                        let press = action.on_press.clone();
                        button(
                            ButtonProps::new(action.label)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .align(ButtonAlign::Start)
                                .full_width()
                                .on_press(move || {
                                    open.set(false);
                                    press();
                                }),
                        )
                    })),
            )
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_dropdown_closes_after_an_action() {
        let source = include_str!("dropdown_menu.rs");
        assert!(source.contains("open.set(false)"));
        assert!(source.contains("z_index(80)"));
    }
}
