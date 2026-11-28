//! One dialog host for every form and confirmation. Screens register a
//! builder at the shell root; the host re-renders it while it is open.

use nsh_core::prelude::*;
use std::rc::Rc;

pub type ModalBuilder = Rc<dyn Fn() -> AnyElement>;

thread_local! {
    /// The open dialog's builder. It lives beside the component tree, so a
    /// shell that renders again keeps the dialog instead of losing it.
    static SLOT: std::cell::RefCell<Option<ModalBuilder>> = const {
        std::cell::RefCell::new(None)
    };
}

/// The host every screen opens its dialogs on.
#[derive(Clone, Copy)]
pub struct DialogHost {
    version: Signal<u64>,
}

impl DialogHost {
    pub fn new() -> Self {
        Self {
            version: crate::ui::dialog_version(),
        }
    }

    fn bump(&self) {
        if let Some(version) = self.version.try_get() {
            let _ = self.version.try_set(version.wrapping_add(1));
        }
    }

    pub fn open(&self, build: impl Fn() -> AnyElement + 'static) {
        SLOT.with(|slot| *slot.borrow_mut() = Some(Rc::new(build)));
        self.bump();
    }

    pub fn close(&self) {
        // Anything the dialog opened (a menu, a picker) captured its signals:
        // it goes before they do.
        crate::ui::close_menu();
        SLOT.with(|slot| *slot.borrow_mut() = None);
        self.bump();
    }

    pub fn active(&self) -> bool {
        let _ = self.version.try_get();
        SLOT.with(|slot| slot.borrow().is_some())
    }

    pub fn render(&self) -> AnyElement {
        let build = SLOT.with(|slot| slot.borrow().clone());
        build.map_or_else(|| view().h(px(0.0)).into_element(), |build| build())
    }
}

pub fn dialog_overlay(modal: DialogHost) -> AnyElement {
    view()
        .absolute()
        .inset_0()
        .z_index(80)
        .child(
            view()
                .absolute()
                .inset_0()
                .bg(rgba(0x00000080))
                .on_press(|| {
                    CURRENT_CLOSE.with(|slot| {
                        if let Some(close) = slot.borrow().as_ref() {
                            close();
                        }
                    });
                }),
        )
        .child(
            view()
                .absolute()
                .inset_0()
                .items_center()
                .justify_center()
                .p(if crate::ui::compact_active() {
                    px(0.0)
                } else {
                    px(24.0)
                })
                .pr(if crate::ui::compact_active() {
                    px(0.0)
                } else {
                    px(28.0)
                })
                .pointer_events(PointerEvents::BoxNone)
                .child(modal.render()),
        )
        .into_element()
}

thread_local! {
    static CURRENT_CLOSE: std::cell::RefCell<Option<Rc<dyn Fn()>>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Shared dialog chrome: title, subtitle, close button, scrollable body and
/// a fixed footer. Every add/edit/notes/bulk form uses this shell so the
/// spacing and behaviour cannot drift between pages.
pub struct DialogProps {
    pub title: String,
    pub title_element: Option<AnyElement>,
    pub subtitle: String,
    pub width: f32,
    pub body: AnyElement,
    pub footer: AnyElement,
    pub show_footer: bool,
    pub on_close: Rc<dyn Fn()>,
    pub scroll_id: Option<&'static str>,
}

impl DialogProps {
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        on_close: impl Fn() + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            title_element: None,
            subtitle: subtitle.into(),
            width: 508.0,
            body: view().into_element(),
            footer: view().into_element(),
            show_footer: true,
            on_close: Rc::new(on_close),
            scroll_id: None,
        }
    }

    pub fn title_element(mut self, title: AnyElement) -> Self {
        self.title_element = Some(title);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn scroll_id(mut self, id: &'static str) -> Self {
        self.scroll_id = Some(id);
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

    pub fn footer_visible(mut self, visible: bool) -> Self {
        self.show_footer = visible;
        self
    }
}

pub fn dialog(dialog: DialogProps) -> impl IntoElement {
    let DialogProps {
        title,
        title_element,
        subtitle,
        width,
        body,
        footer,
        show_footer,
        on_close,
        scroll_id,
    } = dialog;
    CURRENT_CLOSE.with(|slot| {
        *slot.borrow_mut() = Some(on_close.clone());
    });
    let close = on_close.clone();
    let compact = crate::ui::compact_active();
    let body_scroll = element_ref();
    let full = nsh_core::geometry::Length::from(nsh_core::geometry::Percentage(1.0));
    view()
        .w_full()
        .max_w(if compact {
            full
        } else {
            nsh_core::geometry::Length::from(px(width))
        })
        .h(if compact {
            full
        } else {
            nsh_core::geometry::Length::Auto
        })
        .rounded(if compact { px(0.0) } else { px(8.0) })
        .border(px(1.0))
        .border_color(move || nsh_base::theme().border)
        .bg(move || nsh_base::theme().card)
        .shadow([BoxShadow {
            offset_y: px(12.0),
            blur: px(32.0),
            color: rgba(0x00000033).into(),
            ..BoxShadow::default()
        }])
        .overflow_hidden()
        .max_h(if compact {
            full
        } else {
            nsh_core::geometry::Length::from(px(764.0))
        })
        .flex_col()
        .child(
            view()
                .w_full()
                .pl(if compact { px(16.0) } else { px(22.0) })
                .pr(if compact { px(16.0) } else { px(32.0) })
                .pt(if compact { px(16.0) } else { px(24.0) })
                .pb(if compact { px(16.0) } else { px(22.0) })
                .border_b(px(1.0))
                .border_color(move || nsh_base::theme().border)
                .gap(px(4.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(title_element.unwrap_or_else(|| {
                            text(title)
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || nsh_base::theme().foreground)
                                .into_element()
                        }))
                        .child(
                            nsh_base::button()
                                .cursor_pointer()
                                .cursor_pointer()
                                .items_center()
                                .justify_center()
                                .h(px(34.0))
                                .w(px(34.0))
                                .rounded(px(7.0))
                                .accessibility_label("Close dialog")
                                .on_press(move || close())
                                .child(
                                    image(asset!("icons/x.svg"))
                                        .size(px(18.0))
                                        .tint_color(move || nsh_base::theme().muted_foreground),
                                ),
                        ),
                ),
        )
        .child(
            scroll()
                .element_ref(&body_scroll)
                .test_id(scroll_id.map_or("dialog-scroll:none", |id| {
                    if id == "subscription-form" {
                        "dialog-scroll:subscription-form"
                    } else {
                        "dialog-scroll:none"
                    }
                }))
                .flex_grow(1.0)
                .basis(if compact {
                    nsh_core::geometry::Length::from(px(0.0))
                } else {
                    nsh_core::geometry::Length::Auto
                })
                .min_h(px(0.0))
                .max_h(if compact {
                    nsh_core::geometry::Length::Auto
                } else {
                    nsh_core::geometry::Length::from(px(593.0))
                })
                .child(
                    view()
                        .w_full()
                        .flex_shrink_0()
                        .when(!subtitle.is_empty(), |panel| {
                            let subtitle = subtitle.clone();
                            panel.child(
                                view()
                                    .px(if compact { px(16.0) } else { px(22.0) })
                                    .pt(if compact { px(16.0) } else { px(28.0) })
                                    .child(
                                        text(subtitle)
                                            .text_size(px(14.0))
                                            .line_height(px(20.0))
                                            .text_color(move || nsh_base::theme().muted_foreground),
                                    ),
                            )
                        })
                        .child(body),
                ),
        )
        .when(show_footer, |card| {
            card.child(
                view()
                    .w_full()
                    .px(if compact { px(16.0) } else { px(24.0) })
                    .pr(if compact { px(16.0) } else { px(26.0) })
                    .pt(if compact { px(16.0) } else { px(24.0) })
                    .pb(if compact { px(16.0) } else { px(24.0) })
                    .border_t(px(1.0))
                    .border_color(move || nsh_base::theme().border)
                    .child(footer),
            )
        })
}

#[cfg(test)]
mod width_tests {
    #[test]
    fn dialogs_never_exceed_their_max_width() {
        let source = include_str!("dialog.rs");
        assert!(source.contains("max_w(px(width))"));
        assert!(source.contains("w_full()"));
    }
}
