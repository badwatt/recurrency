use dioxus::prelude::*;
use crate::state::auth::AUTH;

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
    // Hydrate auth state from localStorage on mount
    use_future(move || async move {
        if !AUTH().initialized {
            AUTH.write().hydrate_from_storage();
            // Always mark initialized so route guard doesn't loop forever
            AUTH.write().initialized = true;
        }
    });

    rsx! {
        dioxus::document::Stylesheet { href: "/tailwind.css" }
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
    rsx! { div { "Dashboard page" } }
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