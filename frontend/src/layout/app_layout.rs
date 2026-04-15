use dioxus::prelude::*;
use crate::hooks::use_swipe_sidebar::use_swipe_sidebar;

#[component]
pub fn AppLayout() -> Element {
    let mut sidebar_open = use_swipe_sidebar();

    // Auto-close sidebar on route change
    let current_route: crate::Route = use_route();
    use_effect(move || {
        let _ = current_route;
        sidebar_open.set(false);
    });

    rsx! {
        div { class: "flex h-screen bg-background",
            // Desktop sidebar (always visible on md+)
            aside { class: "hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0 border-r border-border bg-card",
                NavSidebar {}
            }

            // Mobile sidebar overlay
            if sidebar_open() {
                div {
                    class: "fixed inset-0 z-40 bg-black/50 md:hidden",
                    id: "mobile-sidebar-backdrop",
                    onclick: move |_| sidebar_open.set(false),
                }
            }

            // Mobile sidebar panel (slides from left)
            aside {
                class: if sidebar_open() { "fixed inset-y-0 left-0 z-50 w-64 bg-card border-r border-border md:hidden transition-transform duration-200" } else { "fixed inset-y-0 left-0 z-50 w-64 bg-card border-r border-border md:hidden -translate-x-full transition-transform duration-200" },
                id: "mobile-sidebar",
                onclick: move |e| e.stop_propagation(),
                NavSidebarMobile { on_close: move |_| sidebar_open.set(false) }
            }

            // Main content area
            div { class: "md:pl-64 flex flex-col flex-1",
                // Mobile header
                header { class: "md:hidden sticky top-0 z-30 flex items-center h-16 px-4 border-b border-border bg-card",
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:text-foreground hover:bg-accent",
                        onclick: move |_| sidebar_open.set(!sidebar_open()),
                        // Hamburger icon (three lines)
                        svg {
                            view_box: "0 0 24 24",
                            width: "24",
                            height: "24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line { x1: "3", y1: "6", x2: "21", y2: "6" }
                            line { x1: "3", y1: "12", x2: "21", y2: "12" }
                            line { x1: "3", y1: "18", x2: "21", y2: "18" }
                        }
                        span { class: "ml-2 text-lg font-bold text-foreground", "Recurrency" }
                    }
                }
                main { class: "flex-1 overflow-y-auto p-6",
                    Outlet::<crate::Route> {}
                }
            }
        }
    }
}

/// Desktop sidebar — no close callback needed
#[component]
fn NavSidebar() -> Element {
    rsx! {
        div { class: "flex flex-col h-full",
            // Logo
            div { class: "flex h-16 items-center px-6 border-b border-border",
                span { class: "text-lg font-bold text-foreground", "Recurrency" }
            }
            // Nav links
            nav { class: "flex-1 px-4 py-4 space-y-1",
                NavItem { to: crate::Route::Dashboard {}, label: "Dashboard", icon: "📊" }
                NavItem { to: crate::Route::Subscriptions {}, label: "Subscriptions", icon: "💳" }
                NavItem { to: crate::Route::Transactions {}, label: "Transactions", icon: "🧾" }
                NavItem { to: crate::Route::Calendar {}, label: "Calendar", icon: "📅" }
                NavItem { to: crate::Route::Categories {}, label: "Categories", icon: "🏷" }
                NavItem { to: crate::Route::Reports {}, label: "Reports", icon: "📈" }
                NavItem { to: crate::Route::Settings {}, label: "Settings", icon: "⚙" }
            }
            // User section
            div { class: "px-4 py-4 border-t border-border",
                p { class: "text-sm text-muted-foreground",
                    {crate::state::auth::AUTH().user.as_ref().map(|u| u.name.clone().unwrap_or_default()).unwrap_or_default()}
                }
            }
        }
    }
}

/// Mobile sidebar — auto-closes on nav click
#[component]
fn NavSidebarMobile(on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex flex-col h-full",
            // Logo
            div { class: "flex h-16 items-center justify-between px-6 border-b border-border",
                span { class: "text-lg font-bold text-foreground", "Recurrency" }
                button {
                    r#type: "button",
                    class: "text-muted-foreground hover:text-foreground p-1",
                    onclick: move |_| on_close.call(()),
                    "✕"
                }
            }
            // Nav links
            nav { class: "flex-1 px-4 py-4 space-y-1",
                NavItemAutoClose { to: crate::Route::Dashboard {}, label: "Dashboard", icon: "📊", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Subscriptions {}, label: "Subscriptions", icon: "💳", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Transactions {}, label: "Transactions", icon: "🧾", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Calendar {}, label: "Calendar", icon: "📅", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Categories {}, label: "Categories", icon: "🏷", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Reports {}, label: "Reports", icon: "📈", on_close: on_close.clone() }
                NavItemAutoClose { to: crate::Route::Settings {}, label: "Settings", icon: "⚙", on_close: on_close.clone() }
            }
            // User section
            div { class: "px-4 py-4 border-t border-border",
                p { class: "text-sm text-muted-foreground",
                    {crate::state::auth::AUTH().user.as_ref().map(|u| u.name.clone().unwrap_or_default()).unwrap_or_default()}
                }
            }
        }
    }
}

#[component]
fn NavItem(to: crate::Route, label: String, icon: String) -> Element {
    let current: crate::Route = use_route();
    let active = current == to;
    let class = if active {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md bg-accent text-accent-foreground"
    } else {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
    };

    rsx! {
        Link { to: to, class: "{class}",
            span { {icon} }
            span { {label} }
        }
    }
}

#[component]
fn NavItemAutoClose(to: crate::Route, label: String, icon: String, on_close: EventHandler<()>) -> Element {
    let current: crate::Route = use_route();
    let active = current == to;
    let class = if active {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md bg-accent text-accent-foreground"
    } else {
        "flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
    };

    rsx! {
        Link {
            to: to,
            class: "{class}",
            onclick: move |_| on_close.call(()),
            span { {icon} }
            span { {label} }
        }
    }
}