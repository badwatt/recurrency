//! One dialog host for every form and confirmation. Screens register a
//! builder at the shell root; the host re-renders it while it is open.

use nsh_core::prelude::*;
use std::rc::Rc;

pub type ModalBuilder = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy)]
pub struct DialogHost {
    slot: Stored<Option<ModalBuilder>>,
    version: Signal<u64>,
}

impl DialogHost {
    pub fn new() -> Self {
        Self {
            slot: stored(None),
            version: signal(0),
        }
    }

    pub fn open(&self, build: impl Fn() -> AnyElement + 'static) {
        self.slot.replace(Some(Rc::new(build)));
        self.version.set(self.version.get().wrapping_add(1));
    }

    pub fn close(&self) {
        self.slot.replace(None);
        self.version.set(self.version.get().wrapping_add(1));
    }

    pub fn active(&self) -> bool {
        self.version.get();
        self.slot.with(|slot| slot.is_some())
    }

    pub fn render(&self) -> AnyElement {
        self.slot
            .with(|slot| slot.clone())
            .map_or_else(|| view().h(px(0.0)).into_element(), |build| build())
    }
}

pub fn dialog_overlay(modal: DialogHost) -> AnyElement {
    view()
        .absolute()
        .inset_0()
        .z_index(80)
        .items_center()
        .justify_center()
        .p(px(24.0))
        .bg(rgba(0x000000cc))
        .child(modal.render())
        .into_element()
}

/// Shared dialog chrome: title, subtitle, close button, scrollable body and
/// a fixed footer. Every add/edit/notes/bulk form uses this shell so the
/// spacing and behaviour cannot drift between pages.
pub struct DialogProps {
    pub title: String,
    pub subtitle: String,
    pub width: f32,
    pub body: AnyElement,
    pub footer: AnyElement,
    pub on_close: Rc<dyn Fn()>,
}

impl DialogProps {
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        on_close: impl Fn() + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            width: 508.0,
            body: view().into_element(),
            footer: view().into_element(),
            on_close: Rc::new(on_close),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn body(mut self, body: AnyElement) -> Self {
        self.body = body;
        self
    }

    pub fn footer(mut self, footer: AnyElement) -> Self {
        self.footer = footer;
        self
    }
}

pub fn dialog(dialog: DialogProps) -> impl IntoElement {
    let DialogProps {
        title,
        subtitle,
        width,
        body,
        footer,
        on_close,
    } = dialog;
    let close = on_close.clone();
    view()
        .w(px(width))
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || nsh_base::theme().border)
        .bg(move || nsh_base::theme().card)
        .overflow_hidden()
        .max_h(px(700.0))
        .flex_col()
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .pb(px(16.0))
                .border_b(px(1.0))
                .border_color(move || nsh_base::theme().border)
                .gap(px(4.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text(title)
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || nsh_base::theme().foreground),
                        )
                        .child(
                            nsh_base::button()
                                .items_center()
                                .justify_center()
                                .h(px(34.0))
                                .w(px(34.0))
                                .rounded(px(7.0))
                                .border(px(1.0))
                                .border_color(move || nsh_base::theme().border)
                                .accessibility_label("Close dialog")
                                .on_press(move || close())
                                .child(
                                    image(asset!("icons/x.svg"))
                                        .size(px(15.0))
                                        .tint_color(move || nsh_base::theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    text(subtitle)
                        .text_size(px(13.0))
                        .text_color(move || nsh_base::theme().muted_foreground),
                ),
        )
        .child(scroll().max_h(px(480.0)).child(body))
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .pt(px(16.0))
                .border_t(px(1.0))
                .border_color(move || nsh_base::theme().border)
                .child(footer),
        )
}
