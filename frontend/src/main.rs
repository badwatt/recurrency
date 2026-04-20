use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;

mod api;
mod auth;
mod components;
mod hooks;
mod layout;
mod models;
mod pages;
mod state;
mod theme;
mod utils;
#[cfg(test)]
mod android_auth_tests;
#[cfg(test)]
mod android_icon_tests;
#[cfg(test)]
mod android_persistence_tests;
#[cfg(test)]
mod android_theme_tests;
#[cfg(test)]
mod android_setup_tests;
#[cfg(test)]

#[cfg(test)]
mod index_html_tests;

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
                let data_dir: std::path::PathBuf = crate::state::auth::get_app_data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                let url_file: std::path::PathBuf = data_dir.join("recurrency").join("server_url.txt");
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
        dioxus::document::Stylesheet { href: asset!("/assets/tailwind.css") }
        Router::<Route> {}
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
    let nav = navigator();
    nav.push(Route::Dashboard {});
    rsx! { div { "Redirecting..." } }
}