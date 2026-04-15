use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div { class: "flex h-screen bg-background",
            // Sidebar
            aside { class: "hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0 border-r border-border bg-card",
                NavSidebar {}
            }
            // Main content area
            div { class: "md:pl-64 flex flex-col flex-1",
                // Mobile header
                header { class: "md:hidden sticky top-0 z-40 flex items-center h-16 px-4 border-b border-border bg-card",
                    span { "Recurrency" }
                }
                main { class: "flex-1 overflow-y-auto p-6",
                    Outlet::<crate::Route> {}
                }
            }
        }
    }
}

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