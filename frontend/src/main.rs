#![allow(dead_code)]

use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use dioxus::prelude::*;

mod api;
mod auth;
mod components;
mod hooks;
mod layout;
mod models;
mod pages;
mod state;
#[cfg(test)]
mod tests;
mod theme;
mod utils;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    // Public
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/setup")]
    Setup {},

    // Protected (layout with sidebar)
    #[layout(layout::AppLayout)]
        #[route("/")]
        Dashboard {},
        #[route("/subscriptions")]
        Subscriptions {},
        #[route("/transactions")]
        Transactions {},
        #[route("/calendar")]
        Calendar {},
        #[route("/categories")]
        Categories {},
        #[route("/reports")]
        Reports {},
        #[route("/settings")]
        Settings {},
    #[end_layout]

    // Catch all → redirect to /
    #[route("/:..path")]
    CatchAll { path: Vec<String> },
}

const TAILWIND_CSS: Asset = asset!("/assets/styles/tailwind.css", AssetOptions::css().with_static_head(true));


fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Hydrate auth state and preferences from localStorage on mount
    // Also restore saved server URL on Android/desktop (where env var isn't persisted)
    use_future(move || async move {
        if !AUTH().initialized {
            AUTH.write().hydrate_from_storage();
            AUTH.write().initialized = true;
        }
        PREFERENCES.write().hydrate_from_storage();

        // Restore saved server URL from persistent storage.
        // On WASM (Docker): same-origin is used, no restore needed.
        // On Android: read from app data file (dioxus::document::eval hangs on Android).
        // On desktop: read from env var or app data file.
        #[cfg(target_arch = "wasm32")]
        {
            // WASM: localStorage is handled by get_base_url_impl() at call time.
            // No async eval needed here since web_sys localStorage is synchronous.
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop/Android: try env var first, then app data file
            if let Ok(url) = std::env::var("RECURRENCY_API_URL") {
                if !url.is_empty() {
                    crate::api::client::set_base_url(&url);
                    eprintln!("[App] Server URL from env: {}", url);
                }
            } else {
                // Try reading from app data file
                let data_dir: std::path::PathBuf = crate::state::auth::get_app_data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let url_file: std::path::PathBuf =
                    data_dir.join("recurrency").join("server_url.txt");
                if url_file.exists() {
                    if let Ok(url) = std::fs::read_to_string(&url_file) {
                        let url = url.trim().to_string();
                        if !url.is_empty() {
                            crate::api::client::set_base_url(&url);
                            eprintln!("[App] Server URL from file: {}", url);
                        }
                    }
                }
            }
        }
    });

    rsx! {
        // Critical inline CSS prevents FOUC on Android WebView.
        // Android WebView doesn't block render on <link> stylesheet.
        // Wrapper div sets dark background immediately via inline style.
        div {
            style: "background:#FAF9F2;color:#202020;min-height:100dvh;font-family:'Cabinet Grotesk',system-ui,-apple-system,sans-serif;",
            document::Stylesheet { href: TAILWIND_CSS }
            Router::<Route> {}
        }
    }
}

#[component]
fn Login() -> Element {
    rsx! { crate::auth::LoginPage {} }
}

#[component]
fn Register() -> Element {
    rsx! { crate::auth::RegisterPage {} }
}

#[component]
fn Setup() -> Element {
    rsx! { crate::pages::SetupPage {} }
}

#[component]
fn Dashboard() -> Element {
    rsx! { crate::pages::Dashboard {} }
}

#[component]
fn Subscriptions() -> Element {
    crate::pages::subscriptions_page::SubscriptionsPage()
}

#[component]
fn Transactions() -> Element {
    rsx! { crate::pages::TransactionsPage {} }
}

#[component]
fn Calendar() -> Element {
    rsx! { crate::pages::CalendarPage {} }
}

#[component]
fn Categories() -> Element {
    rsx! { crate::pages::CategoriesPage {} }
}

#[component]
fn Reports() -> Element {
    rsx! { crate::pages::ReportsPage {} }
}

#[component]
fn Settings() -> Element {
    rsx! { crate::pages::SettingsPage {} }
}

#[component]
fn CatchAll(path: Vec<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-dvh gap-6 px-4 bg-background",
            div { class: "flex flex-col items-center gap-2",
                h1 { class: "text-7xl font-bold tracking-tighter text-muted-foreground/40", "404" }
                p { class: "text-lg text-muted-foreground", "Page not found" }
                p { class: "text-sm text-muted-foreground/60 max-w-md text-center text-balance",
                    "The page you're looking for doesn't exist or was moved."
                }
            }
            a {
                href: "/",
                class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-all duration-200 bg-primary text-primary-foreground hover:bg-primary/90 active:scale-[0.97] px-6 py-2.5",
                "Back to dashboard"
            }
        }
    }
}
