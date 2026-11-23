//! Shared native shell primitives. They mirror the current Tailwind shapes
//! without copying markup: one navigation bar, one surface, one page header.

use crate::Route;
use crate::auth::AuthSession;
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use std::cell::RefCell;
use uuid::Uuid;

thread_local! {
    static PRIVACY: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    static SETTINGS_INTENT: RefCell<Option<Signal<Option<String>>>> = const { RefCell::new(None) };
    static AI_CHAT: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    static ROUTE: RefCell<Option<Signal<Route>>> = const { RefCell::new(None) };
    static NOTIFICATIONS: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
}

pub fn init_privacy() {
    crate::components::toast::init();
    PRIVACY.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal(false));
        }
    });
    SETTINGS_INTENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal(None));
        }
    });
    AI_CHAT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal(false));
        }
    });
    NOTIFICATIONS.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal(false));
        }
    });
}

pub fn notifications_open() -> bool {
    NOTIFICATIONS.with(|slot| slot.borrow().map_or(false, |open| open.get()))
}

pub fn open_notifications() {
    NOTIFICATIONS.with(|slot| {
        if let Some(open) = *slot.borrow() {
            open.set(true);
        }
    });
}

pub fn close_notifications() {
    NOTIFICATIONS.with(|slot| {
        if let Some(open) = *slot.borrow() {
            open.set(false);
        }
    });
}

pub fn init_route(route: Signal<Route>) {
    ROUTE.with(|slot| {
        *slot.borrow_mut() = Some(route);
    });
}

pub fn navigate(route: Route) {
    ROUTE.with(|slot| {
        if let Some(current) = *slot.borrow() {
            current.set(route);
        }
    });
}

pub fn ai_chat_open() -> bool {
    AI_CHAT.with(|slot| slot.borrow().map_or(false, |open| open.get()))
}

pub fn open_ai_chat() {
    AI_CHAT.with(|slot| {
        if let Some(open) = *slot.borrow() {
            open.set(true);
        }
    });
}

pub fn close_ai_chat() {
    AI_CHAT.with(|slot| {
        if let Some(open) = *slot.borrow() {
            open.set(false);
        }
    });
}

pub fn request_settings(route: Signal<Route>, section: &'static str) {
    SETTINGS_INTENT.with(|slot| {
        if let Some(intent) = *slot.borrow() {
            intent.set(Some(section.to_string()));
        }
    });
    route.set(Route::Settings);
}

pub fn settings_intent() -> Option<String> {
    SETTINGS_INTENT.with(|slot| slot.borrow().and_then(|intent| intent.get()))
}

pub fn clear_settings_intent() {
    SETTINGS_INTENT.with(|slot| {
        if let Some(intent) = *slot.borrow() {
            intent.set(None);
        }
    });
}

pub fn privacy_active() -> bool {
    PRIVACY.with(|slot| slot.borrow().map_or(false, |privacy| privacy.get()))
}

fn toggle_privacy() {
    PRIVACY.with(|slot| {
        if let Some(privacy) = *slot.borrow() {
            privacy.set(!privacy.get());
        }
    });
}

pub fn top_bar(route: Signal<Route>, auth: AuthSession, compact: Signal<bool>) -> impl IntoElement {
    view()
        .h(px(56.0))
        .w_full()
        .flex_row()
        .items_center()
        .justify_center()
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            view()
                .h_full()
                .w_full()
                .max_w(px(1280.0))
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px(px(28.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .mr(px(4.0))
                        .child(
                            image(crate::LOGO)
                                .size(px(20.0))
                                .tint_color(move || theme().primary),
                        )
                        .child(
                            text("Recurrency")
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    when(
                        move || !compact.get(),
                        move || {
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(2.0))
                                .children(Route::NAV.iter().map(|item| nav_item(*item, route)))
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(view().flex_1())
                .child(ai_action(route, auth.clone()))
                .child(notification_action())
                .child(privacy_action())
                .child(icon_action(
                    route,
                    asset!("icons/palette.svg"),
                    "appearance",
                ))
                .child(avatar(route, auth)),
        )
}

fn icon_action(route: Signal<Route>, icon: ImageAsset, section: &'static str) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .w(px(30.0))
        .rounded(px(6.0))
        .on_press(move || request_settings(route, section))
        .child(
            image(icon)
                .size(px(17.0))
                .tint_color(move || theme().muted_foreground),
        )
}

fn ai_action(route: Signal<Route>, auth: AuthSession) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .w(px(30.0))
        .rounded(px(6.0))
        .accessibility_label("AI assistant")
        .on_press(move || open_ai(route, auth.clone()))
        .child(
            image(asset!("icons/sparkles.svg"))
                .size(px(17.0))
                .tint_color(move || theme().muted_foreground),
        )
}

/// Open the assistant when it is configured, otherwise jump to its settings.
pub fn open_ai(route: Signal<Route>, auth: AuthSession) {
    let server = crate::auth::stored_server();
    let token = auth.token;
    spawn(async move {
        match crate::api::load_ai_settings(server, token).await {
            Ok(settings) if settings.enabled && settings.configured => open_ai_chat(),
            _ => request_settings(route, "ai"),
        }
    });
}

fn notification_action() -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .w(px(30.0))
        .rounded(px(6.0))
        .accessibility_label("Notifications")
        .on_press(open_notifications)
        .child(
            image(asset!("icons/bell.svg"))
                .size(px(17.0))
                .tint_color(move || theme().muted_foreground),
        )
}

fn privacy_action() -> impl IntoElement {
    crate::components::privacy_toggle::privacy_toggle(privacy_active(), toggle_privacy)
}

fn avatar(route: Signal<Route>, auth: AuthSession) -> impl IntoElement {
    let name = auth
        .user
        .name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| auth.user.email.clone());
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .w(px(30.0))
        .rounded(px(999.0))
        .on_press(move || route.set(Route::Settings))
        .child(crate::components::avatar::avatar_badge(
            30.0,
            &name,
            auth.user.avatar,
        ))
}

fn nav_item(item: Route, route: Signal<Route>) -> impl IntoElement {
    let active = move || route.get() == item;
    button()
        .items_center()
        .justify_center()
        .h(px(36.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .bg(move || {
            if active() {
                theme().accent
            } else {
                theme().card
            }
        })
        .active(|s| s.bg(move || theme().muted))
        .on_press(move || route.set(item))
        .child(image(item.icon()).size(px(18.0)).tint_color(move || {
            if active() {
                theme().foreground
            } else {
                theme().muted_foreground
            }
        }))
        .child(
            text(item.label())
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if active() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

pub fn bottom_bar(route: Signal<Route>) -> impl IntoElement {
    view()
        .w_full()
        .h(px(60.0))
        .flex_row()
        .items_center()
        .justify_around()
        .border_t(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .children(Route::NAV.iter().map(|item| {
            let item = *item;
            let active = move || route.get() == item;
            button()
                .items_center()
                .justify_center()
                .h(px(48.0))
                .px(px(6.0))
                .rounded(px(8.0))
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.0))
                .on_press(move || route.set(item))
                .child(image(item.icon()).size(px(18.0)).tint_color(move || {
                    if active() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }))
                .child(text(item.label()).text_size(px(9.0)).text_color(move || {
                    if active() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }))
        }))
}

pub fn page_header(title: &'static str, subtitle: &'static str) -> impl IntoElement {
    view()
        .min_w(px(0.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text(title)
                .text_size(px(48.0))
                .line_height(px(48.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(subtitle)
                .text_size(px(16.0))
                .line_height(px(24.0))
                .text_color(move || theme().muted_foreground),
        )
}

pub fn surface() -> nsh_core::element::View {
    crate::components::card::card()
}

pub fn pending_screen(title: &'static str) -> AnyElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(crate::theme::PAGE_GAP))
        .child(page_header(title, "Native Shell migration in progress"))
        .child(
            surface().p(px(24.0)).child(
                text("This screen is being ported from the Dioxus interface.")
                    .text_size(px(15.0))
                    .text_color(move || theme().muted_foreground),
            ),
        )
        .into_element()
}

pub fn count_label(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

pub fn label(text_value: &'static str) -> impl IntoElement {
    text(text_value)
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(move || theme().muted_foreground)
}

pub fn field(
    label_text: &'static str,
    value: Signal<String>,
    placeholder: &'static str,
) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(label(label_text))
        .child(crate::components::input::text_input(value, placeholder))
}

pub fn date_field(label_text: &'static str, value: Signal<String>) -> impl IntoElement {
    crate::components::field::field(
        crate::components::field::FieldProps::new(label_text),
        crate::components::date_picker::date_picker(value).into_element(),
    )
}

pub fn choice_picker(
    label_text: &'static str,
    items: impl Fn() -> Vec<(Uuid, String)> + Clone + 'static,
    selected: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let options = move || {
        items()
            .into_iter()
            .map(|(id, name)| crate::components::select_menu::SelectOption::new(id, name))
            .collect()
    };
    crate::components::select_menu::select_menu(
        crate::components::select_menu::SelectMenuProps::new(
            label_text,
            "Select...",
            selected,
            options,
        ),
    )
}

pub fn action_button(
    label: impl Into<String>,
    destructive: bool,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    crate::components::button::button(
        crate::components::button::ButtonProps::new(label.into())
            .variant(if destructive {
                crate::components::button::ButtonVariant::Destructive
            } else {
                crate::components::button::ButtonVariant::Outline
            })
            .size(crate::components::button::ButtonSize::Xs)
            .radius(crate::components::button::ButtonRadius::Sm)
            .on_press(on_press),
    )
}

pub use crate::components::dialog::DialogHost as ModalHost;

pub fn modal_overlay(modal: ModalHost) -> AnyElement {
    crate::components::dialog::dialog_overlay(modal)
}

pub fn icon_button(
    icon: ImageAsset,
    active: bool,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    crate::components::button::icon_button(icon, "Action", active, on_press)
}

pub fn mini_icon_button(
    icon: ImageAsset,
    destructive: bool,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(26.0))
        .w(px(26.0))
        .rounded(px(5.0))
        .on_press(on_press)
        .child(image(icon).size(px(13.0)).tint_color(move || {
            if destructive {
                theme().destructive
            } else {
                theme().muted_foreground
            }
        }))
}
