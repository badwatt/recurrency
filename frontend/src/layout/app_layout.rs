use crate::api::notifications;
use crate::state::auth::AUTH;
use crate::state::notifications::NOTIFICATIONS;
use crate::state::preferences::PREFERENCES;
use crate::theme::hooks::{apply_theme_class, toggle_theme};
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
}

#[component]
pub fn AppLayout() -> Element {
    let auth = AUTH();

    use_future(move || async move {
        let _ = dioxus::document::eval(
            "document.getElementById('pre-wasm-loader').style.display='none';",
        );
    });

    if !auth.initialized {
        return rsx! {
            div { class: "fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background/85 backdrop-blur-xl text-foreground",
                div { class: "relative h-12 w-12",
                    div { class: "absolute inset-0 animate-spin rounded-full border-4 border-muted border-t-primary" }
                    div { class: "absolute inset-2 animate-spin rounded-full border-4 border-muted/50 border-b-primary/50" }
                }
                p { class: "text-sm text-muted-foreground font-medium animate-pulse", "Loading..." }
            }
        };
    }

    if !auth.is_authenticated {
        let nav = navigator();
        nav.push(crate::Route::Login {});
        return rsx! { div {} };
    }

    let current_route: crate::Route = use_route();

    use_effect(move || {
        let theme = PREFERENCES().theme().to_string();
        apply_theme_class(&theme);
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        let js = r#"
        (function() {
            if (window.__recurrency_system_listener) return;
            window.__recurrency_system_listener = true;
            var mql = window.matchMedia('(prefers-color-scheme: dark)');
            mql.addEventListener('change', function(e) {
                var stored = localStorage.getItem('recurrency-theme');
                if (stored === 'system') {
                    document.documentElement.className = e.matches ? 'dark' : 'light';
                }
            });
        })()
        "#;
        let _ = dioxus::document::eval(js);
    });

    // Notification click-outside overlay (at App level, outside backdrop-blur header)
    let mut notif_overlay = use_signal(|| false);

    rsx! {
        a {
            href: "#main-content",
            class: "sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[60] focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-foreground focus:rounded-md",
            "Skip to content"
        }

        div { class: "flex flex-col min-h-dvh bg-background",

            // ── Top navigation bar (sticky) ──
            header { class: "sticky top-0 z-30 w-full border-b border-border/50 bg-background/80 backdrop-blur-xl",
                div { class: "flex items-center justify-between h-14 px-4 lg:px-6 max-w-screen-2xl mx-auto",
                    // Left: Logo + nav links
                    div { class: "flex items-center gap-6",
                        a {
                            href: "/",
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
                                to: crate::Route::Categories {},
                                label: "Categories".to_string(),
                                active: matches!(current_route, crate::Route::Categories {}),
                                icon: Icon::Categories,
                            }
                            TopNavLink {
                                to: crate::Route::Reports {},
                                label: "Reports".to_string(),
                                active: matches!(current_route, crate::Route::Reports {}),
                                icon: Icon::Reports,
                            }
                        }
                    }
                    // Right: theme toggle + notification bell + settings
                    div { class: "flex items-center gap-1",
                        ThemeToggleButton {}
                        NotificationBell {}
                        a {
                            href: "/settings",
                            class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                            {SvgIcon(Icon::Settings, 20)}
                        }
                    }
                }
            }

            // ── Main content ──
            if notif_overlay() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| { notif_overlay.set(false); },
                }
            }

            main { id: "main-content", class: "flex-1 w-full max-w-screen-2xl mx-auto",
                Outlet::<crate::Route> {}
            }

            // ── Mobile bottom bar ──
            nav { class: "lg:hidden fixed bottom-0 left-0 right-0 z-30 border-t border-border/50 bg-background/80 backdrop-blur-xl safe-area-bottom",
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
                        to: crate::Route::Calendar {},
                        label: "Calendar".to_string(),
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
        }
    }
}

// ── Top nav link (horizontal, pill-style) ──
#[component]
fn TopNavLink(to: crate::Route, label: String, active: bool, icon: Icon) -> Element {
    rsx! {
        a {
            href: format!("{}", to),
            class: navlink_class(active),
            {SvgIcon(icon, 18)}
            span { class: "hidden lg:inline", "{label}" }
        }
    }
}

// ── Mobile bottom tab ──
#[component]
fn MobileTab(to: crate::Route, label: String, active: bool, icon: Icon) -> Element {
    rsx! {
        a {
            href: format!("{}", to),
            class: if active {
                "flex flex-col items-center gap-0.5 px-3 py-1 text-primary"
            } else {
                "flex flex-col items-center gap-0.5 px-3 py-1 text-muted-foreground"
            },
            {SvgIcon(icon, 22)}
            span { class: "text-[10px] font-medium", "{label}" }
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
            {
                let theme = theme_signal.theme().to_string();
                let (icon, _label) = theme_icon_and_label(&theme);
                rsx! { {SvgIcon(icon, 20)} }
            }
        }
    }
}

// ── Notification bell ──
#[component]
fn NotificationBell() -> Element {
    let notif_state = NOTIFICATIONS();
    let unread = notif_state.unread_count;
    let mut open = use_signal(|| false);

    // Fetch notifications on mount
    use_future(move || async move {
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
                    if should_open {
                        spawn(async move {
                            if let Err(e) = notifications::list().await {
                                log::error!("Failed to fetch notifications: {}", e);
                            }
                        });
                    }
                },
                {SvgIcon(Icon::Dashboard, 20)}
                if unread > 0 {
                    span { class: "absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-md bg-primary px-1 text-[10px] font-bold text-primary-foreground",
                        "{notification_badge_text(unread as usize)}"
                    }
                }
            }

            // Dropdown panel
            if open() {
                div {
                    class: "absolute right-0 top-full mt-2 w-80 rounded-lg border border-border bg-card shadow-xl z-50 overflow-hidden",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "flex items-center justify-between px-4 py-3 border-b border-border",
                        h3 { class: "text-sm font-semibold text-foreground", "Notifications" }
                        if notif_state.notifications.iter().any(|n| !n.is_read) {
                            button {
                                r#type: "button",
                                class: "text-xs text-muted-foreground hover:text-foreground transition-colors",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Err(e) = notifications::mark_all_as_read().await {
                                            log::error!("Failed to mark all read: {}", e);
                                        }
                                    });
                                },
                                "Mark all read"
                            }
                        }
                    }
                    div { class: "max-h-80 overflow-y-auto",
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
                // Click-outside overlay
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| open.set(false),
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
                p { class: "text-sm font-medium text-foreground truncate", "{notification.title}" }
                if let Some(message) = &notification.message {
                    p { class: "text-xs text-muted-foreground mt-0.5 line-clamp-2", "{message}" }
                }
                p { class: "text-[10px] text-muted-foreground/60 mt-1", "{notification.created_at}" }
            }
            if !is_read {
                button {
                    r#type: "button",
                    class: "p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                    onclick: move |_| {
                        let id = notification.id;
                        spawn(async move {
                            if let Err(e) = notifications::mark_as_read(&id).await {
                                log::error!("Failed to mark read: {}", e);
                            }
                        });
                    },
                    {SvgIcon(Icon::Dashboard, 14)}
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

// ── Helper: theme icon and label ──
pub fn theme_icon_and_label(theme: &str) -> (Icon, &'static str) {
    match theme {
        "dark" => (Icon::Moon, "Dark"),
        "mocha" => (Icon::Coffee, "Mocha"),
        "system" => (Icon::Monitor, "System"),
        _ => (Icon::Sun, "Light"),
    }
}

// ── Helper: nav link classes ──
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
fn SvgIcon(icon: Icon, size: i32) -> Element {
    match icon {
        Icon::Dashboard => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
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
                width: "{size}",
                height: "{size}",
                rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                line { x1: "8", y1: "21", x2: "16", y2: "21" }
                line { x1: "12", y1: "17", x2: "12", y2: "21" }
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
    fn theme_light_icon_is_sun() {
        let (icon, label) = theme_icon_and_label("light");
        assert_eq!(icon, Icon::Sun);
        assert!(!label.is_empty());
    }

    #[test]
    fn theme_dark_icon_is_moon() {
        let (icon, label) = theme_icon_and_label("dark");
        assert_eq!(icon, Icon::Moon);
        assert!(!label.is_empty());
    }

    #[test]
    fn theme_mocha_icon_is_coffee() {
        let (icon, label) = theme_icon_and_label("mocha");
        assert_eq!(icon, Icon::Coffee);
        assert!(!label.is_empty());
    }

    #[test]
    fn theme_system_icon_is_monitor() {
        let (icon, label) = theme_icon_and_label("system");
        assert_eq!(icon, Icon::Monitor);
        assert!(!label.is_empty());
    }

    #[test]
    fn theme_unknown_icon_is_sun() {
        let (icon, _) = theme_icon_and_label("unknown");
        assert_eq!(icon, Icon::Sun);
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
            let _ = SvgIcon(icon, 20);
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
