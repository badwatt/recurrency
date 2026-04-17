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

        // Restore saved server URL from localStorage (persists across app restarts)
        // On WASM (Docker), same-origin is used — no need to restore.
        // On Android/desktop, the user explicitly configured a URL.
        #[cfg(not(target_arch = "wasm32"))]
        {
            use dioxus::document::eval;
            eprintln!("[App] Restoring server URL from localStorage...");
            match eval("localStorage.getItem('RECURRENCY_API_URL') || ''").await {
                Ok(result) => {
                    eprintln!("[App] localStorage result: {:?}", result);
                    if let Some(url) = result.as_str() {
                        if !url.is_empty() {
                            crate::api::client::set_base_url(url);
                            eprintln!("[App] Server URL restored: {}", url);
                        } else {
                            eprintln!("[App] No server URL in localStorage");
                        }
                    }
                }
                Err(e) => eprintln!("[App] localStorage read failed: {}", e),
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