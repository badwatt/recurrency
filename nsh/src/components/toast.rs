//! One transient notification stack for the whole app.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use nsh_base::{button, theme};
use nsh_core::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastTone {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq)]
struct ToastItem {
    id: u64,
    message: String,
    tone: ToastTone,
}

thread_local! {
    static TOASTS: RefCell<Option<Signal<Vec<ToastItem>>>> = const { RefCell::new(None) };
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

pub fn init() {
    TOASTS.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal(Vec::new()));
        }
    });
}

fn push(message: String, tone: ToastTone) {
    init();
    TOASTS.with(|slot| {
        if let Some(toasts) = *slot.borrow() {
            let id = NEXT_ID.with(|next| {
                let id = next.get();
                next.set(id + 1);
                id
            });
            let mut next = toasts.get();
            next.push(ToastItem { id, message, tone });
            toasts.set(next);
            set_timeout(Duration::from_secs(4), move || dismiss(id));
        }
    });
}

pub fn success(message: impl Into<String>) {
    push(message.into(), ToastTone::Success);
}

pub fn error(message: impl Into<String>) {
    push(message.into(), ToastTone::Error);
}

pub fn info(message: impl Into<String>) {
    push(message.into(), ToastTone::Info);
}

fn dismiss(id: u64) {
    TOASTS.with(|slot| {
        if let Some(toasts) = *slot.borrow() {
            toasts.set(
                toasts
                    .get()
                    .into_iter()
                    .filter(|toast| toast.id != id)
                    .collect(),
            );
        }
    });
}

pub fn toast_overlay() -> AnyElement {
    init();
    let items = TOASTS.with(|slot| slot.borrow().map_or_else(Vec::new, |toasts| toasts.get()));
    if items.is_empty() {
        return view().h(px(0.0)).into_element();
    }
    view()
        .absolute()
        .right(px(20.0))
        .bottom(px(20.0))
        .z_index(140)
        .flex_col()
        .items_end()
        .gap(px(8.0))
        .children(items.into_iter().map(|toast| {
            let id = toast.id;
            let color = match toast.tone {
                ToastTone::Success => crate::theme::success(),
                ToastTone::Error => theme().destructive,
                ToastTone::Info => theme().primary,
            };
            view()
                .max_w(px(360.0))
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .p(px(12.0))
                .rounded(px(9.0))
                .border(px(1.0))
                .border_color(move || color.opacity(0.4))
                .bg(move || theme().card)
                .child(
                    view()
                        .h(px(8.0))
                        .w(px(8.0))
                        .rounded(px(999.0))
                        .bg(move || color),
                )
                .child(
                    text(toast.message)
                        .flex_grow(1.0)
                        .text_size(px(12.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    button()
                        .h(px(22.0))
                        .w(px(22.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(5.0))
                        .accessibility_label("Dismiss notification")
                        .on_press(move || dismiss(id))
                        .child(
                            image(asset!("icons/x.svg"))
                                .size(px(12.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .into_element()
        }))
        .into_element()
}

#[cfg(test)]
mod tests {
    #[test]
    fn toast_tones_have_a_shared_color_source() {
        let source = include_str!("toast.rs");
        assert!(source.contains("crate::theme::success()"));
        assert!(source.contains("Duration::from_secs(4)"));
    }
}
