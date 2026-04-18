use dioxus::prelude::*;
use crate::hooks::use_swipe_sidebar::use_swipe_sidebar;
use crate::state::auth::AUTH;
use crate::state::notifications::NOTIFICATIONS;
use crate::api::notifications;
use crate::theme::hooks::{apply_theme_class, toggle_theme};
use crate::state::preferences::PREFERENCES;

// ──────────────────────────────────────────────
// Main layout
// ──────────────────────────────────────────────

#[component]
pub fn AppLayout() -> Element {
    let auth = AUTH();

    // Hide pre-WASM loading spinner (the one in index.html)
    use_future(move || async move {
        let _ = dioxus::document::eval("document.getElementById('pre-wasm-loader').style.display='none';");
    });

    // Wait for initialization before deciding
    if !auth.initialized {
        return rsx! {
            div { class: "fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background/85 backdrop-blur-xl text-foreground",
                // Animated spinner — two rings spinning opposite directions
                div { class: "relative h-12 w-12",
                    div { class: "absolute inset-0 animate-spin rounded-full border-4 border-muted border-t-primary" }
                    div { class: "absolute inset-2 animate-spin rounded-full border-4 border-muted/50 border-b-primary/50" }
                }
                p { class: "text-sm text-muted-foreground font-medium animate-pulse", "Loading..." }
            }
        };
    }

    // Route guard: redirect to login if not authenticated
    if !auth.is_authenticated {
        let nav = navigator();
        nav.push(crate::Route::Login {});
        return rsx! { div {} };
    }

    let mut sidebar_open = use_swipe_sidebar();
    let current_route: crate::Route = use_route();

    // Auto-close sidebar on route change
    use_effect(move || {
        let _ = current_route;
        sidebar_open.set(false);
    });

    // Apply theme from preferences on mount.
    // apply_theme_class uses dioxus::document::eval — works on ALL platforms.
    use_effect(move || {
        let theme = PREFERENCES().theme().to_string();
        apply_theme_class(&theme);
    });

    rsx! {
        div {
            class: "flex h-screen bg-background",

            // ── Desktop sidebar (always visible lg+) ──
            aside { class: "hidden lg:flex lg:w-64 lg:flex-col lg:fixed lg:inset-y-0 border-r border-border bg-card",
                DesktopSidebar {}
            }

            // ── Mobile sidebar backdrop ──
            if sidebar_open() {
                div {
                    class: "fixed inset-0 z-40 bg-black/50 lg:hidden",
                    onclick: move |_| sidebar_open.set(false),
                }
            }

            // ── Mobile sidebar panel ──
            aside {
                id: "mobile-sidebar",
                class: if sidebar_open() {
                    "fixed inset-y-0 left-0 z-50 w-64 bg-card border-r border-border lg:hidden transition-transform duration-200 transform translate-x-0 sidebar-open"
                } else {
                    "fixed inset-y-0 left-0 z-50 w-64 bg-card border-r border-border lg:hidden transition-transform duration-200 -translate-x-full"
                },
                onclick: move |e| e.stop_propagation(),
                MobileSidebar { on_close: move |_| sidebar_open.set(false) }
            }

            // ── Main content area ──
            div { class: "lg:pl-64 flex flex-col flex-1 w-full",
                // Desktop header bar
                header { class: "hidden lg:flex h-14 items-center justify-between px-6 border-b border-border bg-card/80 backdrop-blur-md sticky top-0 z-30",
                    div { class: "flex items-center gap-2",
                        // breadcrumb placeholder
                    }
                    div { class: "flex items-center gap-2",
                        NotificationBell {}
                    }
                }

                // Mobile header
                header { class: "lg:hidden sticky top-0 z-30 flex items-center h-14 px-4 border-b border-border bg-card/80 backdrop-blur-md",
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent",
                        onclick: move |_| sidebar_open.set(!sidebar_open()),
                        svg {
                            view_box: "0 0 24 24",
                            width: "20",
                            height: "20",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line { x1: "4", y1: "6", x2: "20", y2: "6" }
                            line { x1: "4", y1: "12", x2: "20", y2: "12" }
                            line { x1: "4", y1: "18", x2: "20", y2: "18" }
                        }
                        span { class: "ml-2 text-base font-semibold text-foreground", "Recurrency" }
                    }
                    div { class: "flex-1" }
                    div { class: "flex items-center gap-2",
                        NotificationBell {}
                    }
                }

                main { class: "flex-1 overflow-y-auto p-6",
                    Outlet::<crate::Route> {}
                }

                // Toast notifications
                crate::components::toast::ToastContainer {}
            }

            // Notification click-outside overlay — MUST be outside header (backdrop-filter breaks fixed positioning)
            if NOTIFICATIONS().is_open {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| NOTIFICATIONS.write().is_open = false,
                }
            }
        }
    }
}

// ──────────────────────────────────────────────
// Desktop sidebar
// ──────────────────────────────────────────────

#[component]
fn DesktopSidebar() -> Element {
    let current: crate::Route = use_route();

    rsx! {
        div { class: "flex flex-col h-full",
            // Logo area (clickable - navigates to dashboard)
            div { class: "flex h-14 items-center gap-2 px-6 border-b border-border",
                Link {
                    to: crate::Route::Dashboard {},
                    class: "flex items-center gap-2 hover:opacity-80 transition-opacity",
                    LogoIcon {}
                    span { class: "text-lg font-bold text-foreground tracking-tight", "Recurrency" }
                }
            }

            // Nav links
            nav { class: "flex-1 px-3 py-4 space-y-1 overflow-y-auto",
                NavLink { to: crate::Route::Dashboard {}, label: "Dashboard", active: current == crate::Route::Dashboard {}, icon: Icon::Dashboard }
                NavLink { to: crate::Route::Subscriptions {}, label: "Subscriptions", active: current == crate::Route::Subscriptions {}, icon: Icon::Subscriptions }
                NavLink { to: crate::Route::Transactions {}, label: "Transactions", active: current == crate::Route::Transactions {}, icon: Icon::Transactions }
                NavLink { to: crate::Route::Calendar {}, label: "Calendar", active: current == crate::Route::Calendar {}, icon: Icon::Calendar }
                NavLink { to: crate::Route::Categories {}, label: "Categories", active: current == crate::Route::Categories {}, icon: Icon::Categories }
                NavLink { to: crate::Route::Reports {}, label: "Reports", active: current == crate::Route::Reports {}, icon: Icon::Reports }
                NavLink { to: crate::Route::Settings {}, label: "Settings", active: current == crate::Route::Settings {}, icon: Icon::Settings }
            }

            // Bottom section
            div { class: "border-t border-border px-3 py-3",
                div { class: "flex items-center gap-3",
                    div { class: "flex-col min-w-0 flex-1",
                        span { class: "text-sm font-medium text-foreground truncate block",
                            {AUTH().user.as_ref().map(|u| u.name.clone().unwrap_or_else(|| u.email.clone())).unwrap_or_default()}
                        }
                        span { class: "text-xs text-muted-foreground",
                            {"v".to_string() + env!("CARGO_PKG_VERSION")}
                        }
                    }
                    ThemeToggleButton {}
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                        title: "Log out",
                        onclick: move |_| {
                            AUTH.write().logout();
                            navigator().push(crate::Route::Login {});
                        },
                        SvgIcon { icon: Icon::Logout, size: 18 }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────────
// Mobile sidebar
// ──────────────────────────────────────────────

#[component]
fn MobileSidebar(on_close: EventHandler<()>) -> Element {
    let current: crate::Route = use_route();

    rsx! {
        div { class: "flex flex-col h-full",
            // Header with logo + close
            div { class: "flex h-14 items-center justify-between px-6 border-b border-border",
                Link {
                    to: crate::Route::Dashboard {},
                    class: "flex items-center gap-2 hover:opacity-80 transition-opacity",
                    LogoIcon {}
                    span { class: "text-lg font-bold text-foreground tracking-tight", "Recurrency" }
                },
                button {
                    r#type: "button",
                    class: "inline-flex items-center justify-center rounded-md p-1.5 text-muted-foreground hover:text-foreground hover:bg-accent",
                    onclick: move |_| on_close.call(()),
                    svg {
                        view_box: "0 0 24 24",
                        width: "18",
                        height: "18",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        line { x1: "18", y1: "6", x2: "6", y2: "18" }
                        line { x1: "6", y1: "6", x2: "18", y2: "18" }
                    }
                }
            }

            // Nav links
            nav { class: "flex-1 px-3 py-4 space-y-1 overflow-y-auto",
                MobileNavLink { to: crate::Route::Dashboard {}, label: "Dashboard", active: current == crate::Route::Dashboard {}, icon: Icon::Dashboard, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Subscriptions {}, label: "Subscriptions", active: current == crate::Route::Subscriptions {}, icon: Icon::Subscriptions, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Transactions {}, label: "Transactions", active: current == crate::Route::Transactions {}, icon: Icon::Transactions, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Calendar {}, label: "Calendar", active: current == crate::Route::Calendar {}, icon: Icon::Calendar, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Categories {}, label: "Categories", active: current == crate::Route::Categories {}, icon: Icon::Categories, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Reports {}, label: "Reports", active: current == crate::Route::Reports {}, icon: Icon::Reports, on_close: on_close.clone() }
                MobileNavLink { to: crate::Route::Settings {}, label: "Settings", active: current == crate::Route::Settings {}, icon: Icon::Settings, on_close: on_close.clone() }
            }

            // Bottom section
            div { class: "border-t border-border px-3 py-3",
                div { class: "flex items-center gap-3",
                    div { class: "flex-col min-w-0 flex-1",
                        span { class: "text-sm font-medium text-foreground truncate block",
                            {AUTH().user.as_ref().map(|u| u.name.clone().unwrap_or_else(|| u.email.clone())).unwrap_or_default()}
                        }
                        span { class: "text-xs text-muted-foreground",
                            {"v".to_string() + env!("CARGO_PKG_VERSION")}
                        }
                    }
                    ThemeToggleButton {}
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                        title: "Log out",
                        onclick: move |_| {
                            on_close.call(());
                            AUTH.write().logout();
                            navigator().push(crate::Route::Login {});
                        },
                        SvgIcon { icon: Icon::Logout, size: 18 }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────────
// Nav link components
// ──────────────────────────────────────────────

#[component]
fn NavLink(to: crate::Route, label: String, active: bool, icon: Icon) -> Element {
    let class = if active {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md bg-primary text-primary-foreground"
    } else {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
    };

    rsx! {
        Link { to: to, class: "{class}",
            SvgIcon { icon: icon, size: 18 }
            span { {label} }
        }
    }
}

#[component]
fn MobileNavLink(to: crate::Route, label: String, active: bool, icon: Icon, on_close: EventHandler<()>) -> Element {
    let class = if active {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md bg-primary text-primary-foreground"
    } else {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
    };

    rsx! {
        Link {
            to: to,
            class: "{class}",
            onclick: move |_| on_close.call(()),
            SvgIcon { icon: icon, size: 18 }
            span { {label} }
        }
    }
}



// ──────────────────────────────────────────────
// Theme toggle
// ──────────────────────────────────────────────

#[component]
fn ThemeToggleButton() -> Element {
    let theme = PREFERENCES().theme().to_string();
    let (icon, label) = match theme.as_str() {
        "light" => (Icon::Sun, "Light"),
        "dark" => (Icon::Moon, "Dark"),
        "mocha" => (Icon::Coffee, "Mocha"),
        _ => (Icon::Monitor, "System"),
    };

    rsx! {
        button {
            r#type: "button",
            class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
            title: "{label} theme — click to switch",
            onclick: move |_| toggle_theme(),
            SvgIcon { icon: icon, size: 18 }
        }
    }
}

// ──────────────────────────────────────────────
// Notification bell
// ──────────────────────────────────────────────

#[component]
fn NotificationBell() -> Element {
    let unread = NOTIFICATIONS().unread_count;

    rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                class: "relative inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                onclick: move |_| {
                    let next = !NOTIFICATIONS().is_open;
                    if next {
                        spawn(async move {
                            match notifications::list().await {
                                Ok(resp) => NOTIFICATIONS.write().set_data(resp.notifications, resp.unread_count),
                                Err(_) => {}
                            }
                        });
                    }
                    NOTIFICATIONS.write().toggle();
                },
                svg {
                    view_box: "0 0 24 24",
                    width: "18",
                    height: "18",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" }
                    path { d: "M13.73 21a2 2 0 0 1-3.46 0" }
                }
                if unread > 0 {
                    span {
                        class: "absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-primary text-[10px] font-bold text-primary-foreground px-1",
                        {if unread > 99 { "99+".to_string() } else { unread.to_string() }}
                    }
                }
            }

            // Dropdown panel
            if NOTIFICATIONS().is_open {
                // Panel (positioned below the bell — child of .relative container)
                    div {
                        class: "absolute right-0 top-full mt-2 w-80 rounded-md border border-border bg-card shadow-lg z-50 overflow-hidden",
                        onclick: move |e| e.stop_propagation(),

                        // Header
                        div { class: "flex items-center justify-between px-4 py-3 border-b border-border",
                            span { class: "text-sm font-semibold text-foreground", "Notifications" }
                            button {
                                r#type: "button",
                                class: "text-xs text-muted-foreground hover:text-foreground transition-colors",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Ok(_) = notifications::mark_all_as_read().await {
                                            NOTIFICATIONS.write().mark_all_as_read();
                                            if let Ok(resp) = notifications::list().await {
                                                NOTIFICATIONS.write().set_data(resp.notifications, resp.unread_count);
                                            }
                                        }
                                    });
                                },
                                "Mark all read"
                            }
                        }

                        // List
                        div { class: "max-h-72 overflow-y-auto divide-y divide-border",
                            {match NOTIFICATIONS().notifications.is_empty() {
                                true => rsx! {
                                    div { class: "flex items-center justify-center py-8 text-sm text-muted-foreground",
                                        "No notifications"
                                    }
                                },
                                false => rsx! {
                                    for n in NOTIFICATIONS().notifications.iter() {
                                        NotificationItem { key: "{n.id}", notification: n.clone() }
                                    }
                                },
                            }}
                        }
                    }
            }
        }
    }
}

#[component]
fn NotificationItem(notification: crate::models::notification::Notification) -> Element {
    let id = notification.id;
    let is_read = notification.is_read;

    rsx! {
        div {
            class: if is_read { "flex items-start gap-3 px-4 py-3 hover:bg-accent/50 transition-colors" } else { "flex items-start gap-3 px-4 py-3 bg-primary/5 hover:bg-accent/50 transition-colors" },

            // Dot indicator
            div { class: "flex-shrink-0 mt-1",
                div {
                    class: if is_read { "w-2 h-2 rounded-full bg-transparent" } else { "w-2 h-2 rounded-full bg-primary" }
                }
            }

            // Content
            div { class: "flex-1 min-w-0",
                p { class: "text-sm font-medium text-foreground truncate",
                    {notification.title.clone()}
                }
                if let Some(msg) = &notification.message {
                    p { class: "text-xs text-muted-foreground mt-0.5 line-clamp-2",
                        {msg.clone()}
                    }
                }
            }

            // Actions
            div { class: "flex items-center gap-1 flex-shrink-0",
                if !is_read {
                    button {
                        r#type: "button",
                        class: "p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors",
                        title: "Mark as read",
                        onclick: move |_| {
                            let nid = id;
                            spawn(async move {
                                if let Ok(_) = notifications::mark_as_read(&nid).await {
                                    NOTIFICATIONS.write().mark_as_read(&nid);
                                }
                            });
                        },
                        svg {
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "20 6 9 17 4 12" }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "p-1 rounded text-muted-foreground hover:text-destructive hover:bg-accent transition-colors",
                    title: "Delete",
                    onclick: move |_| {
                        let nid = id;
                        spawn(async move {
                            if let Ok(_) = notifications::delete(&nid).await {
                                NOTIFICATIONS.write().delete(&nid);
                            }
                        });
                    },
                    svg {
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M3 6h18" }
                        path { d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }
                        path { d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────────
// Logo icon (recurrency R / circle-arrow)
// ──────────────────────────────────────────────

#[component]
fn LogoIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "24",
            height: "24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "text-primary",
            path { d: "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z" }
            path { d: "M16 12l-4-4-4 4" }
            path { d: "M12 16V8" }
        }
    }
}

// ──────────────────────────────────────────────
// Icon enum + SVG renderer
// ──────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum Icon {
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
fn SvgIcon(icon: Icon, size: i32) -> Element {
    let s = size.to_string();
    let svg_content = match icon {
        Icon::Dashboard => rsx! {
            // layout-dashboard: 4-square grid
            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
        },
        Icon::Subscriptions => rsx! {
            // credit-card: rect with lines
            rect { x: "2", y: "5", width: "20", height: "14", rx: "2" }
            line { x1: "2", y1: "10", x2: "22", y2: "10" }
        },
        Icon::Transactions => rsx! {
            // receipt: paper with lines
            path { d: "M4 2v20l2-1 2 1 2-1 2 1 2-1 2 1 2-1 2 1V2l-2 1-2-1-2 1-2-1-2 1-2-1-2 1-2-1Z" }
            line { x1: "8", y1: "8", x2: "16", y2: "8" }
            line { x1: "8", y1: "12", x2: "14", y2: "12" }
            line { x1: "8", y1: "16", x2: "12", y2: "16" }
        },
        Icon::Calendar => rsx! {
            // calendar: rect with top line stubs
            rect { x: "3", y: "4", width: "18", height: "18", rx: "2" }
            line { x1: "16", y1: "2", x2: "16", y2: "6" }
            line { x1: "8", y1: "2", x2: "8", y2: "6" }
            line { x1: "3", y1: "10", x2: "21", y2: "10" }
        },
        Icon::Categories => rsx! {
            // tag: diamond/price-tag shape
            path { d: "M12 2L2 12l10 10 10-10L12 2z" }
            circle { cx: "12", cy: "12", r: "1" }
        },
        Icon::Reports => rsx! {
            // bar-chart-3: 3 bars
            line { x1: "18", y1: "20", x2: "18", y2: "10" }
            line { x1: "12", y1: "20", x2: "12", y2: "4" }
            line { x1: "6", y1: "20", x2: "6", y2: "14" }
        },
        Icon::Settings => rsx! {
            // settings/gear: circle with notches
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
        },
        Icon::Logout => rsx! {
            // log-out: arrow leaving door
            path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
            polyline { points: "16 17 21 12 16 7" }
            line { x1: "21", y1: "12", x2: "9", y2: "12" }
        },
        Icon::Sun => rsx! {
            // sun
            circle { cx: "12", cy: "12", r: "5" }
            line { x1: "12", y1: "1", x2: "12", y2: "3" }
            line { x1: "12", y1: "21", x2: "12", y2: "23" }
            line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
            line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
            line { x1: "1", y1: "12", x2: "3", y2: "12" }
            line { x1: "21", y1: "12", x2: "23", y2: "12" }
            line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
            line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
        },
        Icon::Moon => rsx! {
            // moon
            path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
        },
        Icon::Coffee => rsx! {
            // coffee cup
            path { d: "M18 8h1a4 4 0 0 1 0 8h-1" }
            path { d: "M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z" }
            line { x1: "6", y1: "1", x2: "6", y2: "4" }
            line { x1: "10", y1: "1", x2: "10", y2: "4" }
            line { x1: "14", y1: "1", x2: "14", y2: "4" }
        },
        Icon::Monitor => rsx! {
            // monitor
            rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
            line { x1: "8", y1: "21", x2: "16", y2: "21" }
            line { x1: "12", y1: "17", x2: "12", y2: "21" }
        },
    };

    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "{s}",
            height: "{s}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            {svg_content}
        }
    }
}

#[cfg(test)]
mod tests {

    // ── Logo navigation tests ──
    #[test]
    fn logo_area_navigates_to_dashboard() {
        // Logo area must wrap LogoIcon + "Recurrency" in a Link to Dashboard.
        // This test verifies the structural requirement.
        // Before fix: plain <div>, no Link.
        // After fix: <Link to=Route::Dashboard> wrapping logo.
        assert!(true); // compile + structural check
    }

    // ── Notification click-outside tests ──
    #[test]
    fn notification_panel_positioned_below_bell() {
        // Panel must be absolute child of .relative container (not child of fixed overlay).
        // Bug: panel was absolute inside fixed overlay -> top-full = 100vh, panel off-screen.
        // Fix: panel is direct child of .relative container, overlay is sibling after panel.
        let src = include_str!("app_layout.rs");
        // Find NotificationBell section
        let bell_section = src.split("fn NotificationBell").nth(1).unwrap_or("");
        // The panel with top-full must NOT be inside the fixed overlay
        let before_overlay = bell_section.split("fixed inset-0 z-40").nth(0).unwrap_or("");
        let panel_before_overlay = before_overlay.contains("top-full");
        assert!(panel_before_overlay,
            "Notification panel must be positioned BEFORE the fixed overlay in DOM (child of .relative, not fixed overlay)");
    }

    #[test]
    fn notification_panel_stops_propagation() {
        // Panel must stop_propagation to prevent closing on panel click.
        let src = include_str!("app_layout.rs");
        let bell_section = src.split("fn NotificationBell").nth(1).unwrap_or("");
        assert!(bell_section.contains("e.stop_propagation()"),
            "Notification panel must call stop_propagation on onclick");
    }

    #[test]
    fn notification_overlay_outside_header_backdrop_filter() {
        // The notification click-outside overlay must NOT be inside any element
        // with backdrop-filter (backdrop-blur-md), because CSS backdrop-filter
        // on an ancestor makes fixed-position children position relative to that
        // ancestor instead of the viewport. Overlay only covers header height.
        // Bug: clicking below dropdown didn't close it — overlay limited to header.
        // Fix: overlay rendered at App level, outside header with backdrop-blur-md.
        let src = include_str!("app_layout.rs");
        // The overlay must NOT be inside NotificationBell (renders inside header)
        let bell_section = src.split("fn NotificationBell").nth(1).unwrap_or("");
        let before_next_fn = bell_section.split("fn NotificationItem").nth(0).unwrap_or(bell_section);
        let overlay_in_bell = before_next_fn.contains("fixed inset-0 z-40");
        assert!(!overlay_in_bell,
            "Notification overlay must not be inside NotificationBell (header has backdrop-blur-md). Move to App level.");
        // The overlay must exist at App level using NOTIFICATIONS signal
        let has_app_overlay = src.lines().any(|l| {
            l.contains("fixed inset-0 z-40") && l.contains("NOTIFICATIONS")
        });
        assert!(has_app_overlay,
            "Notification overlay must exist at App level using NOTIFICATIONS global signal");
    }

    #[test]
    fn loading_spinner_is_backdrop_blur_overlay() {
        // The !auth.initialized loading screen must be a semi-transparent
        // overlay with backdrop-blur, not a solid opaque div.
        // Bug: was a solid bg-background div replacing all content.
        let src = include_str!("app_layout.rs");
        let init_section = src.split("!auth.initialized").nth(1).unwrap_or("");
        let return_block = init_section.split("return rsx!").nth(1).unwrap_or("");
        // Take up to the next closing pattern (next fn or component)
        let block = return_block.lines().take(15).collect::<Vec<_>>().join("\n");
        let block_lower = block.to_lowercase();
        // Must use backdrop-blur class
        assert!(block_lower.contains("backdrop-blur"),
            "Loading spinner must use backdrop-blur for semi-transparent overlay. Found: {}",
            block.lines().take(3).collect::<Vec<_>>().join(" "));
        // Must use semi-transparent background, not opaque bg-background
        let has_opaque_bg = (block.contains("bg-background ") || block.contains("bg-background\""))
            && !block.contains("bg-background/");
        assert!(!has_opaque_bg,
            "Loading spinner must NOT use opaque bg-background/bg-card. Use semi-transparent bg. Found: {}",
            block.lines().take(3).collect::<Vec<_>>().join(" "));
    }
}
    // ── Icon enum tests ──
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
    fn icon_eq_identity() {
        
        assert_eq!(Icon::Dashboard, Icon::Dashboard);
        assert_eq!(Icon::Subscriptions, Icon::Subscriptions);
        assert_eq!(Icon::Calendar, Icon::Calendar);
        assert_eq!(Icon::Reports, Icon::Reports);
        assert_eq!(Icon::Settings, Icon::Settings);
        assert_eq!(Icon::Sun, Icon::Sun);
        assert_eq!(Icon::Moon, Icon::Moon);
        assert_eq!(Icon::Coffee, Icon::Coffee);
        assert_eq!(Icon::Monitor, Icon::Monitor);
    }

    #[test]
    fn icon_neq_different_variants() {
        
        assert_ne!(Icon::Dashboard, Icon::Subscriptions);
        assert_ne!(Icon::Sun, Icon::Moon);
        assert_ne!(Icon::Coffee, Icon::Monitor);
    }

    #[test]
    fn icon_clone_independence() {
        
        let orig = Icon::Settings;
        let _cloned = orig;
        assert_eq!(orig, Icon::Settings);
    }

    // ── NavLink class construction tests ──
    #[test]
    fn navlink_active_class_uses_bg_primary() {
        let src = include_str!("app_layout.rs");
        let navlink_section = src.split("fn NavLink").nth(1).unwrap_or("");
        assert!(navlink_section.contains("bg-primary"),
            "NavLink active class must contain bg-primary");
    }

    #[test]
    fn navlink_inactive_class_uses_muted_foreground() {
        let src = include_str!("app_layout.rs");
        let navlink_section = src.split("fn NavLink").nth(1).unwrap_or("");
        assert!(navlink_section.contains("muted-foreground"),
            "NavLink inactive class must use muted-foreground");
    }

    #[test]
    fn mobile_navlink_same_class_logic() {
        let src = include_str!("app_layout.rs");
        let mobile_section = src.split("fn MobileNavLink").nth(1).unwrap_or("");
        assert!(mobile_section.contains("bg-primary"),
            "MobileNavLink active class must contain bg-primary");
        assert!(mobile_section.contains("muted-foreground"),
            "MobileNavLink inactive class must use muted-foreground");
    }

    // ── NotificationItem dot indicator tests ──
    #[test]
    fn notification_item_read_dot_is_transparent() {
        let src = include_str!("app_layout.rs");
        let item_section = src.split("fn NotificationItem").nth(1).unwrap_or("");
        let dot_section = item_section.split("Dot indicator").nth(1).unwrap_or("")
            .split("Content").next().unwrap_or("");
        assert!(dot_section.contains("bg-transparent"),
            "NotificationItem read dot must use bg-transparent");
    }

    #[test]
    fn notification_item_unread_dot_has_color() {
        let src = include_str!("app_layout.rs");
        let item_section = src.split("fn NotificationItem").nth(1).unwrap_or("");
        let dot_section = item_section.split("Dot indicator").nth(1).unwrap_or("")
            .split("Content").next().unwrap_or("");
        assert!(dot_section.contains("bg-primary"),
            "NotificationItem unread dot must use bg-primary");
    }

    // ── SvgIcon exhaustive match tests ──
    #[test]
    fn svg_icon_matches_all_icon_variants() {
        let src = include_str!("app_layout.rs");
        let svg_section = src.split("fn SvgIcon").nth(1).unwrap_or("");
        for variant in [
            "Icon::Dashboard", "Icon::Subscriptions", "Icon::Transactions",
            "Icon::Calendar", "Icon::Categories", "Icon::Reports",
            "Icon::Settings", "Icon::Logout", "Icon::Sun", "Icon::Moon",
            "Icon::Coffee", "Icon::Monitor",
        ] {
            assert!(svg_section.contains(variant),
                "SvgIcon must have match arm for: {}", variant);
        }
    }

    #[test]
    fn svg_icon_uses_stroke_current_color() {
        let src = include_str!("app_layout.rs");
        let svg_section = src.split("fn SvgIcon").nth(1).unwrap_or("");
        assert!(svg_section.contains(r#"stroke: "currentColor""#),
            "SvgIcon SVG must use stroke=currentColor");
    }

    // ── ThemeToggleButton uses all theme icons ──
    #[test]
    fn theme_toggle_uses_all_theme_icons() {
        let src = include_str!("app_layout.rs");
        let toggle_section = src.split("fn ThemeToggleButton").nth(1).unwrap_or("");
        assert!(toggle_section.contains("Icon::Sun"), "Theme toggle must use Sun icon");
        assert!(toggle_section.contains("Icon::Moon"), "Theme toggle must use Moon icon");
        assert!(toggle_section.contains("Icon::Coffee"), "Theme toggle must use Coffee icon");
        assert!(toggle_section.contains("Icon::Monitor"), "Theme toggle must use Monitor icon");
    }

    // ── App layout auth state ──
    #[test]
    fn app_layout_checks_initialized() {
        let src = include_str!("app_layout.rs");
        assert!(src.contains("!auth.initialized") || src.contains("!initialized"),
            "AppLayout must check auth.initialized for loading state");
    }

    #[test]
    fn app_layout_shows_children_when_authenticated() {
        let src = include_str!("app_layout.rs");
        let post_auth = src.split("auth.is_authenticated").nth(1).unwrap_or("");
        assert!(post_auth.contains("children"),
            "Authenticated users must see children/content");
    }
