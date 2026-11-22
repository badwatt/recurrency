//! Shared destructive confirmation row and dialog.

use nsh_base::theme;
use nsh_core::prelude::*;
use std::rc::Rc;

pub struct ConfirmProps {
    pub message: String,
    pub confirm_label: String,
    pub on_confirm: Rc<dyn Fn()>,
    pub on_cancel: Rc<dyn Fn()>,
}

impl ConfirmProps {
    pub fn new(
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        on_confirm: impl Fn() + 'static,
        on_cancel: impl Fn() + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            confirm_label: confirm_label.into(),
            on_confirm: Rc::new(on_confirm),
            on_cancel: Rc::new(on_cancel),
        }
    }
}

pub fn confirm_row(props: ConfirmProps) -> impl IntoElement {
    let ConfirmProps {
        message,
        confirm_label,
        on_confirm,
        on_cancel,
    } = props;
    let confirm = on_confirm.clone();
    view()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(6.0))
        .child(
            text(message)
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_row()
                .gap(px(4.0))
                .child(crate::ui::action_button(
                    confirm_label.clone(),
                    true,
                    move || confirm(),
                ))
                .child(crate::ui::action_button("Cancel", false, move || {
                    on_cancel()
                })),
        )
}

pub fn confirm_dialog(props: ConfirmProps) -> impl IntoElement {
    let ConfirmProps {
        message,
        confirm_label,
        on_confirm,
        on_cancel,
    } = props;
    let cancel = on_cancel.clone();
    let confirm = on_confirm.clone();
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new("Are you sure?", message, move || cancel())
            .width(420.0)
            .body(
                view()
                    .p(px(24.0))
                    .child(
                        text("This action cannot be undone.")
                            .text_size(px(12.0))
                            .text_color(move || theme().muted_foreground),
                    )
                    .into_element(),
            )
            .footer(
                view()
                    .w_full()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.0))
                    .child(crate::ui::action_button("Cancel", false, move || {
                        on_cancel()
                    }))
                    .child(crate::ui::action_button(
                        confirm_label.clone(),
                        true,
                        move || confirm(),
                    ))
                    .into_element(),
            ),
    )
}
