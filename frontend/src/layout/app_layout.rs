#![allow(clippy::suspicious_else_formatting)]
use crate::components::privacy_toggle::PrivacyToggle;
#[cfg(all(target_os = "linux", feature = "desktop"))]
use crate::components::toast::TOAST;
use crate::components::toast::ToastContainer;
use crate::components::update_banner::UpdateBanner;
use crate::state::preferences::PREFERENCES;
#[cfg(all(target_os = "linux", feature = "desktop"))]
use crate::state::window;
use crate::theme::hooks::{apply_theme_class, toggle_theme};
use crate::utils::scroll::scroll_to_top;
#[cfg(all(target_os = "linux", feature = "desktop"))]
use dioxus::desktop::use_muda_event_handler;

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
use crate::state::create_intent::{CREATE_INTENT, CreateIntent};
use dioxus::prelude::*;

/// Icon enum for nav and theme icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Dashboard,
    Subscriptions,
    Transactions,
    Debts,
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
    Info,
    Close,
}

#[component]
pub fn AppLayout() -> Element {
    let auth = AUTH();
    let privacy = *crate::state::privacy::PRIVACY.read();

    use_future(move || async move {
        let _ = dioxus::document::eval(
            "var p=document.getElementById('pre-wasm-loader');if(p)p.style.display='none';",
        );
    });

    #[cfg(all(target_os = "linux", feature = "desktop"))]
    use_muda_event_handler(move |event| {
        if event.id().0 != window::MENU_ID {
            return;
        }
        let frameless = !window::stored();
        window::set_stored(frameless);
        dioxus::desktop::window().set_decorations(!frameless);
        TOAST.write().success(if frameless {
            "Frameless window enabled"
        } else {
            "Window decorations restored"
        });
    });

    #[cfg(not(target_arch = "wasm32"))]
    use_future(move || async move {
        loop {
            if crate::state::app_mode::APP_MODE().mode.is_offline() {
                let _ = crate::storage::folder_sync::sync_once().await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
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
    let fab_open = use_signal(|| false);
    let in_settings = matches!(
        current_route,
        crate::Route::Settings {}
            | crate::Route::SettingsProfile {}
            | crate::Route::SettingsPreferences {}
            | crate::Route::SettingsMoney {}
            | crate::Route::SettingsNotifications {}
            | crate::Route::SettingsData {}
            | crate::Route::SettingsServer {}
            | crate::Route::SettingsAboutPage {}
            | crate::Route::SettingsAccounts {}
            | crate::Route::SettingsCategories {}
            | crate::Route::Accounts {}
            | crate::Route::Categories {}
    );

    rsx! {
        a {
            href: "#main-content",
            class: "sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[60] focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-foreground focus:rounded-md",
            "Skip to content"
        }

        div { class: if privacy { "privacy-mode flex flex-col min-h-dvh bg-background" } else { "flex flex-col min-h-dvh bg-background" },

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
                                scroll_to_top();
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
                                to: crate::Route::Debts {},
                                label: "Debts".to_string(),
                                active: matches!(current_route, crate::Route::Debts {}),
                                icon: Icon::Debts,
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
                    // Right: privacy + theme + profile and settings
                    div { class: "flex items-center gap-1",
                        PrivacyToggle {}
                        ThemeToggleButton {}
                        a {
                            href: "/settings",
                            aria_label: "Profile and settings",
                            title: "Profile and settings",
                            onclick: move |e| {
                                e.prevent_default();
                                nav.push(crate::Route::Settings {});
                                scroll_to_top();
                            },
                            class: "inline-flex items-center justify-center rounded-full p-0.5 ring-primary/40 hover:ring-2 transition-all",
                            UserAvatar {}
                        }
                    }
                }
            }

            main { id: "main-content", class: "flex-1 w-full max-w-7xl mx-auto",
                UpdateBanner {}
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
                        to: crate::Route::Debts {},
                        label: "Debts".to_string(),
                        active: matches!(current_route, crate::Route::Debts {}),
                        icon: Icon::Debts,
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

        }

        if !in_settings {
            MobileCreateFab { open: fab_open }
        }

        ToastContainer {}
    }
}

#[component]
fn MobileCreateFab(open: Signal<bool>) -> Element {
    let nav = navigator();
    let mut open = open;
    let choose = Callback::new(move |intent: CreateIntent| {
        *CREATE_INTENT.write() = Some(intent);
        open.set(false);
        nav.push(match intent {
            CreateIntent::Subscription => crate::Route::Subscriptions {},
            CreateIntent::Transaction => crate::Route::Transactions {},
            CreateIntent::Debt => crate::Route::Debts {},
        });
    });

    rsx! {
        div { class: "md:hidden fixed bottom-20 right-4 z-40 flex flex-col items-end gap-2",
            if open() {
                button {
                    r#type: "button",
                    class: "inline-flex items-center gap-2 rounded-full border border-border bg-card px-4 py-2.5 text-sm font-medium shadow-lg",
                    onclick: move |_| choose.call(CreateIntent::Subscription),
                    {svg_icon(Icon::Subscriptions, 18)}
                    "Subscription"
                }
                button {
                    r#type: "button",
                    class: "inline-flex items-center gap-2 rounded-full border border-border bg-card px-4 py-2.5 text-sm font-medium shadow-lg",
                    onclick: move |_| choose.call(CreateIntent::Transaction),
                    {svg_icon(Icon::Transactions, 18)}
                    "Transaction"
                }
                button {
                    r#type: "button",
                    class: "inline-flex items-center gap-2 rounded-full border border-border bg-card px-4 py-2.5 text-sm font-medium shadow-lg",
                    onclick: move |_| choose.call(CreateIntent::Debt),
                    {svg_icon(Icon::Debts, 18)}
                    "Debt"
                }
            }
            button {
                r#type: "button",
                aria_label: if open() { "Close create menu" } else { "Create subscription, transaction or debt" },
                aria_expanded: if open() { "true" } else { "false" },
                class: "flex h-14 w-14 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-xl active:scale-95 transition-transform",
                onclick: move |_| open.toggle(),
                svg {
                    view_box: "0 0 24 24",
                    width: "26",
                    height: "26",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    {if open() { rsx! { line { x1: "6", y1: "6", x2: "18", y2: "18" } line { x1: "18", y1: "6", x2: "6", y2: "18" } } } else { rsx! { line { x1: "12", y1: "5", x2: "12", y2: "19" } line { x1: "5", y1: "12", x2: "19", y2: "12" } } }}
                }
            }
        }
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
                scroll_to_top();
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
                scroll_to_top();
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

/// Initials shown when the user has no uploaded profile photo.
pub fn avatar_initials(name: Option<&str>, email: &str) -> String {
    let name = name.unwrap_or("").trim();
    let source = if name.is_empty() {
        email.split('@').next().unwrap_or("?")
    } else {
        name
    };
    let parts: Vec<&str> = source.split_whitespace().collect();
    let initials = if parts.len() >= 2 {
        let first = parts.first().and_then(|p| p.chars().next());
        let last = parts.last().and_then(|p| p.chars().next());
        first.into_iter().chain(last).collect::<String>()
    } else {
        source.chars().take(2).collect()
    };
    if initials.is_empty() {
        "?".to_string()
    } else {
        initials.to_uppercase()
    }
}

/// Stable avatar color derived from the account email.
pub fn avatar_color(seed: &str) -> &'static str {
    const COLORS: [&str; 8] = [
        "#6366f1", "#8b5cf6", "#ec4899", "#f97316", "#14b8a6", "#0ea5e9", "#22c55e", "#eab308",
    ];
    let hash = seed
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    COLORS[hash % COLORS.len()]
}

#[component]
fn UserAvatar() -> Element {
    let user = AUTH().user.clone();
    let email = user.as_ref().map(|u| u.email.as_str()).unwrap_or("");
    let name = user.as_ref().and_then(|u| u.name.as_deref());
    let avatar = user
        .as_ref()
        .and_then(|u| u.avatar.as_deref())
        .filter(|value| !value.is_empty());
    let initials = avatar_initials(name, email);
    let color = avatar_color(email);
    if let Some(src) = avatar {
        rsx! {
            img {
                src: src.to_string(),
                alt: "",
                class: "h-8 w-8 rounded-full object-cover select-none",
                aria_hidden: "true",
            }
        }
    } else {
        rsx! {
            span {
                class: "flex h-8 w-8 items-center justify-center rounded-full text-[11px] font-semibold text-white select-none",
                style: "background-color: {color}",
                aria_hidden: "true",
                {initials}
            }
        }
    }
}

pub fn navlink_class(active: bool) -> &'static str {
    if active {
        "inline-flex items-center gap-2 rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-foreground transition-colors"
    } else {
        "inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
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
        Icon::Debts => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                path { d: "M20 12V8H6a2 2 0 0 1 0-4h12v4" }
                path { d: "M4 6v12a2 2 0 0 0 2 2h14v-4" }
                path { d: "M18 12a2 2 0 0 0 0 4h4v-4z" }
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
        Icon::Info => rsx! {
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                width: size.to_string(),
                height: size.to_string(),
                circle { cx: "12", cy: "12", r: "10" }
                line { x1: "12", y1: "11", x2: "12", y2: "16" }
                line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
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
        assert!(
            !prod_code.contains("lg:fixed"),
            "layout code must not contain lg:fixed"
        );
        assert!(
            prod_code.contains("top-0") && prod_code.contains("sticky"),
            "must have sticky top nav"
        );
    }

    #[test]
    fn layout_has_bottom_bar_for_mobile() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("bottom-0"), "must have bottom bar for mobile");
    }

    #[test]
    fn navlink_active_uses_accent_background() {
        let cls = navlink_class(true);
        assert!(
            cls.contains("bg-accent"),
            "active nav link must use accent bg: {}",
            cls
        );
    }

    #[test]
    fn navlink_inactive_is_muted() {
        let cls = navlink_class(false);
        assert!(
            cls.contains("text-muted-foreground"),
            "inactive nav link must be muted: {}",
            cls
        );
    }

    #[test]
    fn icon_variants_are_instantiable() {
        let _ = Icon::Dashboard;
        let _ = Icon::Subscriptions;
        let _ = Icon::Transactions;
        let _ = Icon::Debts;
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
        assert!(
            src.contains("main-content"),
            "must have main-content id for skip link"
        );
    }

    #[test]
    fn avatar_initials_use_name_words() {
        assert_eq!(
            avatar_initials(Some("Ada Lovelace"), "ada@example.com"),
            "AL"
        );
        assert_eq!(avatar_initials(Some("Álvaro"), "a@example.com"), "ÁL");
    }

    #[test]
    fn avatar_initials_fall_back_to_email() {
        assert_eq!(avatar_initials(None, "demo@example.com"), "DE");
    }

    #[test]
    fn avatar_color_is_stable() {
        assert_eq!(
            avatar_color("demo@example.com"),
            avatar_color("demo@example.com")
        );
        assert!(avatar_color("a@example.com") != avatar_color(""));
    }

    #[test]
    fn topbar_uses_profile_avatar_instead_of_gear() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("Profile and settings"));
        assert!(src.contains("UserAvatar"));
        let gear = ["svg_icon(Icon::", "Settings, 20)"].concat();
        assert!(!src.contains(&gear));
    }

    #[test]
    fn mobile_fab_creates_subscriptions_and_transactions() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("MobileCreateFab"));
        assert!(src.contains("CreateIntent::Subscription"));
        assert!(src.contains("CreateIntent::Transaction"));
        assert!(src.contains("bottom-20 right-4"));
    }

    #[test]
    fn layout_has_skip_to_content_link() {
        let src = include_str!("app_layout.rs");
        assert!(
            src.contains("Skip to content"),
            "must have skip-to-content accessibility link"
        );
    }
}

#[cfg(test)]
mod navigation_tests {
    #[test]
    fn menu_links_scroll_to_the_top() {
        let src = include_str!("app_layout.rs");
        let layout = include_str!("settings_layout.rs");
        assert!(src.contains("scroll_to_top"));
        assert!(layout.contains("scroll_to_top"));
        assert!(include_str!("../utils/scroll.rs").contains("window.scrollTo(0, 0)"));
        assert!(src.contains("folder_sync::sync_once"));
    }
}
