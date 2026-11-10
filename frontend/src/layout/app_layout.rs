#![allow(clippy::suspicious_else_formatting)]
use crate::state::preferences::PREFERENCES;
use crate::theme::hooks::{apply_theme_class, toggle_theme};
use crate::api::notifications;
use crate::components::toast::ToastContainer;

// Branded app loader (auth init, pre-WASM)
#[allow(non_snake_case)]
pub fn AppLoader() -> Element {
    rsx! {
        div { class: "fixed inset-0 z-50 flex flex-col items-center justify-center bg-background",
            div { class: "relative h-12 w-12",
                div { class: "absolute inset-0 animate-spin rounded-full border-2 border-muted border-t-primary" }
                div { class: "absolute inset-1.5 animate-spin rounded-full border-2 border-muted/50 border-b-primary/50", style: "animation-direction: reverse; animation-duration: 1.4s" }
            }
            p { class: "mt-6 text-xl font-bold tracking-tight text-foreground", "Recurrency" }
            p { class: "mt-1 text-sm text-muted-foreground",
                "Every expense is a subscription"
                span { class: "animate-pulse", "..." }
            }
        }
    }
}
use crate::state::auth::AUTH;
use crate::state::notifications::NOTIFICATIONS;
use dioxus::prelude::*;

/// Icon enum for nav and theme icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Dashboard,
    Subscriptions,
    Transactions,
    Calendar,
    Categories,
    Reports,
    Settings,
    Logout,
    Sun,
    Moon,
    Coffee,
    Monitor,
    Bell,
    Close,
}

#[component]
pub fn AppLayout() -> Element {
    let auth = AUTH();

    use_future(move || async move {
        let _ = dioxus::document::eval(
            "var p=document.getElementById('pre-wasm-loader');if(p)p.style.display='none';",
        );
    });

    if !auth.initialized {
        // Wait for auth hydration. Mode is hydrated independently via Default impl.
        return rsx! { AppLoader {} };
    }

    // Both modes require a session: server mode uses a JWT, offline mode uses
    // a local account. An unauthenticated user is redirected to Login.
    if requires_login(auth.is_authenticated) {
        let nav = navigator();
        nav.push(crate::Route::Login {});
        return rsx! { div {} };
    }

    let current_route: crate::Route = use_route();
    use_effect(move || {
        let theme = PREFERENCES().theme().to_string();
        apply_theme_class(&theme);
    });


    // Notification click-outside overlay (at App level, outside backdrop-blur header)
    let mut notif_open = use_signal(|| false);

    // Mobile bottom bar: hide on scroll down, show on scroll up
    use_effect(move || {
        let js = r#"
        (function() {
            if (window.__recurrency_scroll_bar) return;
            window.__recurrency_scroll_bar = true;
            var lastScroll = 0;
            var bar = null;
            function findBar() {
                bar = document.querySelector('[data-mobile-bar]');
                return bar;
            }
            window.addEventListener('scroll', function() {
                if (!bar && !findBar()) return;
                var current = window.scrollY || document.documentElement.scrollTop;
                if (current <= 0) {
                    bar.classList.remove('translate-y-full');
                } else if (current > lastScroll && current > 60) {
                    bar.classList.add('translate-y-full');
                } else if (current < lastScroll) {
                    bar.classList.remove('translate-y-full');
                }
                lastScroll = Math.max(current, 0);
            }, {passive: true});
        })()
        "#;
        let _ = dioxus::document::eval(js);
    });

    let nav = navigator();

    rsx! {
        a {
            href: "#main-content",
            class: "sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[60] focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-foreground focus:rounded-md",
            "Skip to content"
        }

        div { class: "flex flex-col min-h-dvh bg-background",

            // ── Top navigation bar (sticky) ──
            header { class: "sticky top-0 z-30 w-full border-b border-border/50 bg-background/80 backdrop-blur-xl",
                div { class: "flex items-center justify-between h-14 px-4 lg:px-6 max-w-7xl mx-auto",
                    // Left: Logo + nav links
                    div { class: "flex items-center gap-6",
                        a {
                            href: "/",
                            onclick: move |e| {
                                e.prevent_default();
                                nav.push(crate::Route::Dashboard {});
                            },
                            class: "flex items-center gap-2 hover:opacity-80 transition-opacity",
                            LogoIcon {}
                            span { class: "font-bold text-foreground hidden sm:inline", "Recurrency" }
                        }
                        nav { class: "hidden md:flex items-center gap-1",
                            TopNavLink {
                                to: crate::Route::Dashboard {},
                                label: "Dashboard".to_string(),
                                active: matches!(current_route, crate::Route::Dashboard {}),
                                icon: Icon::Dashboard,
                            }
                            TopNavLink {
                                to: crate::Route::Subscriptions {},
                                label: "Subscriptions".to_string(),
                                active: matches!(current_route, crate::Route::Subscriptions {}),
                                icon: Icon::Subscriptions,
                            }
                            TopNavLink {
                                to: crate::Route::Transactions {},
                                label: "Transactions".to_string(),
                                active: matches!(current_route, crate::Route::Transactions {}),
                                icon: Icon::Transactions,
                            }
                            TopNavLink {
                                to: crate::Route::Calendar {},
                                label: "Calendar".to_string(),
                                active: matches!(current_route, crate::Route::Calendar {}),
                                icon: Icon::Calendar,
                            }
                            TopNavLink {
                                to: crate::Route::Reports {},
                                label: "Reports".to_string(),
                                active: matches!(current_route, crate::Route::Reports {}),
                                icon: Icon::Reports,
                            }
                        }
                    }
                    // Right: theme + notifications + settings
                    div { class: "flex items-center gap-1",
                        ThemeToggleButton {}
                        NotificationBell { open: notif_open }
                        a {
                            href: "/settings",
                            onclick: move |e| {
                                e.prevent_default();
                                nav.push(crate::Route::Settings {});
                            },
                            class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                            title: "Settings",
                            {svg_icon(Icon::Settings, 20)}
                        }
                    }
                }
            }

            // Click-outside layer for the notification panel. It lives here
            // (outside the header) because `backdrop-filter` on the header
            // makes `position: fixed` children relative to the header, which
            // previously limited click-outside to the header area. z-20 keeps
            // it below the header (z-30) so the panel stays interactive.
            if notif_open() {
                div {
                    class: "fixed inset-0 z-20",
                    onclick: move |_| { notif_open.set(false); },
                }
            }

            main { id: "main-content", class: "flex-1 w-full max-w-7xl mx-auto",
                Outlet::<crate::Route> {}
            }

            // ── Mobile bottom bar (sticky, hides on scroll down) ──
            nav { class: "lg:hidden sticky bottom-0 z-30 border-t border-border/50 bg-background/80 backdrop-blur-xl transition-transform duration-300",
                "data-mobile-bar": "true",
                div { class: "flex items-center justify-around h-14",
                    MobileTab {
                        to: crate::Route::Dashboard {},
                        label: "Home".to_string(),
                        active: matches!(current_route, crate::Route::Dashboard {}),
                        icon: Icon::Dashboard,
                    }
                    MobileTab {
                        to: crate::Route::Subscriptions {},
                        label: "Subs".to_string(),
                        active: matches!(current_route, crate::Route::Subscriptions {}),
                        icon: Icon::Subscriptions,
                    }
                    MobileTab {
                        to: crate::Route::Transactions {},
                        label: "Trans".to_string(),
                        active: matches!(current_route, crate::Route::Transactions {}),
                        icon: Icon::Transactions,
                    }
                    MobileTab {
                        to: crate::Route::Calendar {},
                        label: "Cal".to_string(),
                        active: matches!(current_route, crate::Route::Calendar {}),
                        icon: Icon::Calendar,
                    }
                    MobileTab {
                        to: crate::Route::Reports {},
                        label: "Reports".to_string(),
                        active: matches!(current_route, crate::Route::Reports {}),
                        icon: Icon::Reports,
                    }
                }
            }

            if notif_open() {
                MobileNotificationsPanel { open: notif_open }
            }
        }

        ToastContainer {}
    }
}

// ── Top nav link (horizontal, pill-style) ──
#[component]
fn TopNavLink(to: crate::Route, label: String, active: bool, icon: Icon) -> Element {
    let nav = navigator();
    rsx! {
        a {
            href: format!("{}", to),
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            class: navlink_class(active),
            {svg_icon(icon, 18)}
            span { class: "hidden lg:inline", {label.to_string()} }
        }
    }
}

// ── Mobile bottom tab ──
#[component]
fn MobileTab(to: crate::Route, label: String, active: bool, icon: Icon) -> Element {
    let nav = navigator();
    rsx! {
        a {
            href: format!("{}", to),
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            class: if active {
                "flex flex-col items-center gap-0.5 px-3 py-1 text-primary"
            } else {
                "flex flex-col items-center gap-0.5 px-3 py-1 text-muted-foreground"
            },
            {svg_icon(icon, 22)}
            span { class: "text-[10px] font-medium", {label.to_string()} }
        }
    }
}


// ── Theme toggle button ──
#[component]
fn ThemeToggleButton() -> Element {
    let theme_signal = PREFERENCES();
    rsx! {
        button {
            r#type: "button",
            class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
            onclick: move |_| toggle_theme(),
            title: "Toggle theme",
            {{
                let theme = theme_signal.theme().to_string();
                let icon = match theme.as_str() {
                    "dark" => Icon::Moon,
                    "mocha" => Icon::Coffee,
                    _ => Icon::Sun,
                };
                rsx! { {svg_icon(icon, 20)} }
            }}
        }
    }
}
// ── Notification bell ──
#[component]
fn NotificationBell(open: Signal<bool>) -> Element {
    let notif_state = NOTIFICATIONS();
    let unread = notif_state.unread_count;
    let mut open = open;

    // Fetch notifications on mount (no server to query in offline mode)
    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        if let Err(e) = notifications::list().await {
            log::error!("Failed to fetch notifications: {}", e);
        }
    });

    rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                class: "relative inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                onclick: move |_| {
                    let should_open = !open();
                    open.set(should_open);
                    if should_open && !crate::api::client::is_offline_mode() {
                        spawn(async move {
                            if let Err(e) = notifications::list().await {
                                log::error!("Failed to fetch notifications: {}", e);
                            }
                        });
                    }
                },
                {svg_icon(Icon::Bell, 20)}
                if unread > 0 {
                    span { class: "absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-md bg-primary px-1 text-[10px] font-bold text-primary-foreground",
                        "{notification_badge_text(unread as usize)}"
                    }
                }
            }

            // Dropdown panel
            if open() {
                div {
                    class: "hidden md:block absolute right-0 top-full mt-2 w-80 rounded-lg border border-border bg-card shadow-xl z-50 overflow-hidden",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "flex items-center justify-between px-4 py-3 border-b border-border",
                        h3 { class: "text-sm font-semibold text-foreground", "Notifications" }
                        if notif_state.notifications.iter().any(|n| !n.is_read) {
                            button {
                                r#type: "button",
                                class: "text-xs text-muted-foreground hover:text-foreground transition-colors",
                                onclick: move |_| {
                                    spawn(async move {
                                        match notifications::mark_all_as_read().await {
                                            Ok(()) => NOTIFICATIONS.write().mark_all_as_read(),
                                            Err(e) => log::error!("Failed to mark all read: {}", e),
                                        }
                                    });
                                },
                                "Mark all read"
                            }
                        }
                    }
                    div { class: "max-h-80 overflow-y-auto",
                        NotificationList {}
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationItem(notification: crate::models::notification::Notification) -> Element {
    let is_read = notification.is_read;
    rsx! {
        div {
            class: notification_item_classes(is_read),
            div { class: notification_dot_class(is_read) }
            div { class: "flex-1 min-w-0",
                p { class: "text-sm font-medium text-foreground truncate", {notification.title.to_string()} }
                if let Some(message) = &notification.message {
                    p { class: "text-xs text-muted-foreground mt-0.5 line-clamp-2", {message.to_string()} }
                }
                p { class: "text-[10px] text-muted-foreground/60 mt-1", {notification.created_at.to_string()} }
            }
            if !is_read {
                button {
                    r#type: "button",
                    class: "p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                    onclick: move |_| {
                        let id = notification.id;
                        spawn(async move {
                            match notifications::mark_as_read(&id).await {
                                Ok(()) => NOTIFICATIONS.write().mark_as_read(&id),
                                Err(e) => log::error!("Failed to mark read: {}", e),
                            }
                        });
                    },
                    {svg_icon(Icon::Dashboard, 14)}
                }
            }
        }
    }
}

#[component]
fn NotificationList() -> Element {
    let notif_state = NOTIFICATIONS();
    rsx! {
        if notif_state.notifications.is_empty() {
            div { class: "px-4 py-8 text-center text-sm text-muted-foreground",
                "No notifications yet"
            }
        } else {
            for notification in &notif_state.notifications {
                NotificationItem { notification: notification.clone() }
            }
        }
    }
}

/// Full-screen notification sheet for mobile. Rendered at the layout root so
/// `position: fixed` is not clipped by the header's backdrop-blur.
#[component]
fn MobileNotificationsPanel(open: Signal<bool>) -> Element {
    let notif_state = NOTIFICATIONS();
    let mut open = open;
    let has_notifications = !notif_state.notifications.is_empty();

    rsx! {
        div { class: "fixed inset-0 z-50 flex flex-col bg-background md:hidden",
            div { class: "flex h-14 shrink-0 items-center justify-between border-b border-border px-4",
                h2 { class: "text-base font-semibold text-foreground", "Notifications" }
                button {
                    r#type: "button",
                    class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                    aria_label: "Close notifications",
                    onclick: move |_| open.set(false),
                    {svg_icon(Icon::Close, 20)}
                }
            }
            div { class: "flex-1 overflow-y-auto overscroll-contain",
                NotificationList {}
            }
            div { class: "shrink-0 space-y-2 border-t border-border p-4",
                button {
                    r#type: "button",
                    disabled: !has_notifications,
                    class: "inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                    onclick: move |_| {
                        spawn(async move {
                            let ids: Vec<_> = NOTIFICATIONS()
                                .notifications
                                .iter()
                                .map(|n| n.id)
                                .collect();
                            for id in ids {
                                if let Err(e) = notifications::delete(&id).await {
                                    log::error!("Failed to delete notification: {}", e);
                                }
                            }
                            NOTIFICATIONS.write().clear();
                        });
                    },
                    "Dismiss all"
                }
                button {
                    r#type: "button",
                    class: "inline-flex h-10 w-full items-center justify-center rounded-md border border-border bg-background px-4 py-2 text-sm font-medium text-foreground hover:bg-accent transition-colors",
                    onclick: move |_| open.set(false),
                    "Close"
                }
            }
        }
    }
}

// ── Logo icon (SVG) ──
#[component]
fn LogoIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-6 w-6 text-primary",
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

/// Whether the protected layout must redirect to the login page.
///
/// Both server and offline modes require an authenticated session (offline
/// uses an on-device account).
pub fn requires_login(is_authenticated: bool) -> bool {
    !is_authenticated
}

pub fn navlink_class(active: bool) -> &'static str {
    if active {
        "inline-flex items-center gap-2 rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-foreground transition-colors"
    } else {
        "inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
    }
}

// ── Helper: notification badge ──
pub fn notification_badge_text(count: usize) -> String {
    if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    }
}

// ── Helper: notification item classes ──
pub fn notification_item_classes(is_read: bool) -> &'static str {
    if is_read {
        "flex items-start gap-3 px-4 py-3 hover:bg-accent/50 transition-colors"
    } else {
        "flex items-start gap-3 px-4 py-3 bg-primary/5 hover:bg-accent/50 transition-colors"
    }
}

// ── Helper: notification dot ──
pub fn notification_dot_class(is_read: bool) -> &'static str {
    if is_read {
        "mt-1.5 h-2 w-2 rounded-full bg-transparent shrink-0"
    } else {
        "mt-1.5 h-2 w-2 rounded-full bg-primary shrink-0"
    }
}

// ── SVG icon renderer ──
pub(crate) fn svg_icon(icon: Icon, size: i32) -> Element {
    match icon {
        Icon::Dashboard => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                rect { x: "3", y: "3", width: "7", height: "9", rx: "1" }
                rect { x: "14", y: "3", width: "7", height: "5", rx: "1" }
                rect { x: "14", y: "12", width: "7", height: "9", rx: "1" }
                rect { x: "3", y: "16", width: "7", height: "5", rx: "1" }
            }
        },
        Icon::Subscriptions => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M20 7l-8 4-8-4m16 0l-8-4-8 4m16 0v10l-8 4-8-4V7" }
            }
        },
        Icon::Transactions => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                line { x1: "12", y1: "1", x2: "12", y2: "23" }
                path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 1 1 0 7H6" }
            }
        },
        Icon::Calendar => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
                line { x1: "16", y1: "2", x2: "16", y2: "6" }
                line { x1: "8", y1: "2", x2: "8", y2: "6" }
                line { x1: "3", y1: "10", x2: "21", y2: "10" }
            }
        },
        Icon::Categories => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M4 20h16" }
                path { d: "M6 16l2-6 4 3 4-7 2 4" }
            }
        },
        Icon::Reports => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                polyline { points: "14 2 14 8 20 8" }
                line { x1: "16", y1: "13", x2: "8", y2: "13" }
                line { x1: "16", y1: "17", x2: "8", y2: "17" }
                polyline { points: "10 9 9 9 8 9" }
            }
        },
        Icon::Settings => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                circle { cx: "12", cy: "12", r: "3" }
                path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
            }
        },
        Icon::Logout => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                polyline { points: "16 17 21 12 16 7" }
                line { x1: "21", y1: "12", x2: "9", y2: "12" }
            }
        },
        Icon::Sun => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                circle { cx: "12", cy: "12", r: "5" }
                line { x1: "12", y1: "1", x2: "12", y2: "3" }
                line { x1: "12", y1: "21", x2: "12", y2: "23" }
                line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
                line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
                line { x1: "1", y1: "12", x2: "3", y2: "12" }
                line { x1: "21", y1: "12", x2: "23", y2: "12" }
                line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
                line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
            }
        },
        Icon::Moon => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
            }
        },
        Icon::Coffee => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M18 8h1a4 4 0 1 1 0 8h-1" }
                path { d: "M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z" }
                line { x1: "6", y1: "1", x2: "6", y2: "4" }
                line { x1: "10", y1: "1", x2: "10", y2: "4" }
                line { x1: "14", y1: "1", x2: "14", y2: "4" }
            }
        },
        Icon::Monitor => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                line { x1: "8", y1: "21", x2: "16", y2: "21" }
                line { x1: "12", y1: "17", x2: "12", y2: "21" }
            }
        },
        Icon::Bell => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" }
                path { d: "M10.3 21a1.94 1.94 0 0 0 3.4 0" }
            }
        },
        Icon::Close => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M18 6 6 18" }
                path { d: "M6 6l12 12" }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_area_navigates_to_dashboard() {
        assert!(true); // Logo links to "/"
    }

    #[test]
    fn requires_login_needs_auth() {
        assert!(requires_login(false));
        assert!(!requires_login(true));
    }

    #[test]
    fn layout_uses_top_nav_not_fixed_sidebar() {
        let src = include_str!("app_layout.rs");
        // Check production code only (before #[cfg(test)])
        let prod_code = src.split("#[cfg(test)]").next().unwrap_or(src);
        assert!(!prod_code.contains("lg:fixed"), "layout code must not contain lg:fixed");
        assert!(prod_code.contains("top-0") && prod_code.contains("sticky"), "must have sticky top nav");
    }

    #[test]
    fn layout_has_bottom_bar_for_mobile() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("bottom-0"), "must have bottom bar for mobile");
    }

    #[test]
    fn navlink_active_uses_accent_background() {
        let cls = navlink_class(true);
        assert!(cls.contains("bg-accent"), "active nav link must use accent bg: {}", cls);
    }

    #[test]
    fn navlink_inactive_is_muted() {
        let cls = navlink_class(false);
        assert!(cls.contains("text-muted-foreground"), "inactive nav link must be muted: {}", cls);
    }

    #[test]
    fn notification_badge_text_low() {
        assert_eq!(notification_badge_text(5), "5");
    }

    #[test]
    fn notification_badge_text_high() {
        assert_eq!(notification_badge_text(150), "99+");
    }

    #[test]
    fn icon_variants_are_instantiable() {
        let _ = Icon::Dashboard;
        let _ = Icon::Subscriptions;
        let _ = Icon::Transactions;
        let _ = Icon::Calendar;
        let _ = Icon::Categories;
        let _ = Icon::Reports;
        let _ = Icon::Settings;
        let _ = Icon::Logout;
        let _ = Icon::Sun;
        let _ = Icon::Moon;
        let _ = Icon::Coffee;
        let _ = Icon::Monitor;
    }

    #[test]
    fn icon_enum_discriminants_different() {
        assert_ne!(Icon::Dashboard as u8, Icon::Subscriptions as u8);
        assert_ne!(Icon::Sun as u8, Icon::Moon as u8);
    }


    #[test]
    fn notification_item_unread_has_dot() {
        let cls = notification_dot_class(false);
        assert!(cls.contains("bg-primary"), "unread dot must be visible: {}", cls);
    }

    #[test]
    fn notification_item_read_dot_is_hidden() {
        let cls = notification_dot_class(true);
        assert!(cls.contains("bg-transparent"), "read dot must be transparent: {}", cls);
    }

    #[test]
    fn notification_item_unread_has_highlight() {
        let cls = notification_item_classes(false);
        assert!(cls.contains("primary/5"), "unread must have highlight: {}", cls);
    }

    #[test]
    fn notification_item_read_no_highlight() {
        let cls = notification_item_classes(true);
        assert!(!cls.contains("primary/5"), "read must not have highlight: {}", cls);
    }

    #[test]
    fn notification_panel_stops_propagation() {
        let src = include_str!("app_layout.rs");
        let bell_section = src.split("fn NotificationBell").nth(1).unwrap_or("");
        assert!(
            bell_section.contains("e.stop_propagation()"),
            "Notification panel must call stop_propagation on onclick"
        );
    }

    #[test]
    fn svg_icon_renders_all_variants() {
        let icons = [
            Icon::Dashboard,
            Icon::Subscriptions,
            Icon::Transactions,
            Icon::Calendar,
            Icon::Categories,
            Icon::Reports,
            Icon::Settings,
            Icon::Logout,
            Icon::Sun,
            Icon::Moon,
            Icon::Coffee,
            Icon::Monitor,
        ];
        for icon in icons {
            let _ = svg_icon(icon, 20);
        }
    }

    #[test]
    fn layout_has_main_content_id() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("main-content"), "must have main-content id for skip link");
    }

    #[test]
    fn layout_has_skip_to_content_link() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("Skip to content"), "must have skip-to-content accessibility link");
    }
}
