// Many items are only referenced from `#[cfg(test)]` modules or only on the
// wasm target, so `dead_code` fires on the host library build even though the
// code is used. Keep a crate-level allow rather than annotating ~40 items.
#![allow(dead_code)]
// dioxus-router's `Routable` derive emits code that newer nightly compilers
// flag as `unreachable_code`; it is macro-generated, not ours.
#![allow(unreachable_code)]

use crate::state::app_mode::APP_MODE;
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
mod storage;
#[cfg(test)]
mod tests;
mod theme;
mod utils;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
// Nested layouts end with consecutive `#[end_layout]` markers, which the
// `duplicated_attributes` lint cannot tell apart from a real duplicate.
#[allow(clippy::duplicated_attributes)]
pub enum Route {
    // Public
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/server")]
    ServerSettings {},

    // Protected (layout with sidebar)
    #[layout(layout::AppLayout)]
        #[route("/")]
        Dashboard {},
        #[route("/subscriptions")]
        Subscriptions {},
        #[route("/transactions")]
        Transactions {},
        #[route("/debts")]
        Debts {},
        #[route("/calendar")]
        Calendar {},
        #[route("/reports")]
        Reports {},

        #[layout(layout::SettingsLayout)]
            #[route("/settings")]
            Settings {},
            #[route("/settings/profile")]
            SettingsProfile {},
            #[route("/settings/preferences")]
            SettingsPreferences {},
            #[route("/settings/money")]
            SettingsMoney {},
            #[route("/settings/notifications")]
            SettingsNotifications {},
            #[route("/settings/about")]
            SettingsAboutPage {},
            #[route("/settings/accounts")]
            SettingsAccounts {},
            #[route("/settings/categories")]
            SettingsCategories {},
            #[route("/accounts")]
            Accounts {},
            #[route("/categories")]
            Categories {},
        #[end_layout]
    #[end_layout]

    // Catch all → redirect to /
    #[route("/:..path")]
    CatchAll { path: Vec<String> },
}

const TAILWIND_CSS: Asset = asset!("/assets/styles/tailwind.css", AssetOptions::css().with_static_head(true));

// Self-hosted fonts — registered as Dioxus assets so they're bundled in APK
// Eliminates @import chain to Google Fonts CDN that caused FOUC on Android
const CABINET_GROTESK_400: Asset = asset!("/assets/fonts/cabinet-grotesk-400.woff2");
const CABINET_GROTESK_500: Asset = asset!("/assets/fonts/cabinet-grotesk-500.woff2");
const CABINET_GROTESK_700: Asset = asset!("/assets/fonts/cabinet-grotesk-700.woff2");
const CABINET_GROTESK_800: Asset = asset!("/assets/fonts/cabinet-grotesk-800.woff2");
const JETBRAINS_MONO: Asset = asset!("/assets/fonts/jetbrains-mono-latin.woff2");

// Favicons for the web build.
const FAVICON: Asset = asset!("/assets/icons/clock-32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/icons/clock-256.png");

/// CSS that hides body until styles load — prevents FOUC.
/// Mocha default. Theme class applied by apply_theme_class at runtime.
const FOUC_PREVENTION_CSS: &str = r#"
body { visibility: hidden; }
html { background-color: #1e1e2e; color: #c4d0f0; }
html.light { background-color: #FAF9F2; color: #212121; }
html.dark { background-color: #0d0d11; color: #ededed; }
html.mocha { background-color: #1e1e2e; color: #c4d0f0; }
"#;

/// Generate @font-face CSS using Dioxus-resolved asset URLs.
/// This avoids hardcoded paths that break in Android APK bundles.
fn font_face_css() -> String {
    format!(
        r#"
        @font-face {{ font-family: 'Cabinet Grotesk'; src: url({cg400}) format('woff2'); font-weight: 400; font-display: swap; font-style: normal; }}
        @font-face {{ font-family: 'Cabinet Grotesk'; src: url({cg500}) format('woff2'); font-weight: 500; font-display: swap; font-style: normal; }}
        @font-face {{ font-family: 'Cabinet Grotesk'; src: url({cg700}) format('woff2'); font-weight: 600; font-display: swap; font-style: normal; }}
        @font-face {{ font-family: 'Cabinet Grotesk'; src: url({cg700}) format('woff2'); font-weight: 700; font-display: swap; font-style: normal; }}
        @font-face {{ font-family: 'Cabinet Grotesk'; src: url({cg800}) format('woff2'); font-weight: 800; font-display: swap; font-style: normal; }}
        @font-face {{ font-family: 'JetBrains Mono'; src: url({jm}) format('woff2'); font-weight: 400 700; font-display: swap; font-style: normal; }}
        "#,
        cg400 = CABINET_GROTESK_400,
        cg500 = CABINET_GROTESK_500,
        cg700 = CABINET_GROTESK_700,
        cg800 = CABINET_GROTESK_800,
        jm = JETBRAINS_MONO,
    )
}

/// Application entry point. Called by the thin `main.rs` binary and by `dx`
/// for every platform (web, desktop, Android cdylib).
pub fn launch() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Hydrate auth state, preferences, app mode from storage
    use_future(move || async move {
        if !AUTH().initialized {
            AUTH.write().hydrate_from_storage();
            AUTH.write().initialized = true;
        }
        PREFERENCES.write().hydrate_from_storage();

        // Hydrate app mode from persistent storage
        // APP_MODE hydrates on first read via Default impl, but signal needs explicit write
        let mode_state = APP_MODE();
        if mode_state.initialized {
            // Sync base URL with app mode
            if let Some(url) = mode_state.mode.server_url() {
                crate::api::client::set_base_url(url);
            }
        }

        // A session created in one mode must not leak into the other: an
        // offline session (token "offline") is invalid in server mode and a
        // server JWT is meaningless offline.
        let offline = mode_state.mode.is_offline();
        if let Some(token) = AUTH().token.clone()
            && (token == "offline") != offline
        {
            AUTH.write().logout();
        }

        // Restore saved server URL from persistent storage.
        // On WASM: localStorage handled by get_base_url_impl()
        // On Android/desktop: read from app data file
        #[cfg(target_arch = "wasm32")]
        {
            // WASM: localStorage handled at call time
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
                if url_file.exists()
                    && let Ok(url) = std::fs::read_to_string(&url_file) {
                        let url = url.trim().to_string();
                        if !url.is_empty() {
                            crate::api::client::set_base_url(&url);
                            eprintln!("[App] Server URL from file: {}", url);
                        }
                    }
            }
        }

        // Refresh the cached profile so avatar/name changes made on another
        // device appear without requiring a new login.
        if !offline && AUTH().is_authenticated {
            spawn(async move {
                if let Ok(user) = crate::api::settings::get_user().await {
                    AUTH.write().user = Some(user);
                    AUTH.write().persist_to_storage();
                }
            });
        }
    });

    // Load exchange rates when the session or the main currency changes.
    use_effect(move || {
        let auth = AUTH();
        if auth.is_authenticated && !crate::api::client::is_offline_mode() {
            let base = auth.currency().to_string();
            if crate::state::rates::RATES().base != base {
                spawn(async move {
                    if let Ok(rates) = crate::api::currency::get_rates(&base).await {
                        crate::state::rates::RATES.write().set_data(rates);
                    }
                });
            }
        }
    });

    // A small console Easter egg, visible only in web builds.
    use_future(move || async move {
        #[cfg(target_arch = "wasm32")]
        {
            let version = env!("CARGO_PKG_VERSION");
            dioxus::document::eval(&format!(r#"
                console.log(`%c
   ____
  |  _ \ ___  ___ _   _ _ __ ___ _ __ _   _
  | |_) / _ \/ __| | | | '__/ _ \ '_ \| | | |
  |  _ <  __/ (__| |_| | | |  __/ | | | |_| |
  |_| \_\___|\___|\__,_|_|  \___|_| |_|\__, |
                                       |___/
Recurrency v{version}
Thanks for using Recurrency. If it saves you money, consider starring the project on GitHub.
`, "color:#8b9cff;font-family:monospace;font-size:12px;");
            "#));
        }
    });

    // Show body only after CSS + fonts are fully applied — prevents FOUC on Android.
    // Uses link.onload events instead of sheet.cssRules (which throws SecurityError
    // on Android WebView's custom protocol, causing premature body reveal).
    use_effect(move || {
        dioxus::document::eval(r#"
            (function() {
                function reveal() {
                    // Never let a slow font load keep the native loader forever.
                    var fontsReady = Promise.race([
                        document.fonts.ready,
                        new Promise(function(resolve) { setTimeout(resolve, 900); })
                    ]);
                    fontsReady.then(function() {
                        requestAnimationFrame(function() {
                            requestAnimationFrame(function() {
                                document.body.style.setProperty('visibility', 'visible', 'important');
                                ['pre-wasm-loader', 'dxloader'].forEach(function (id) {
                                    var el = document.getElementById(id);
                                    if (el) el.remove();
                                });
                            });
                        });
                    });
                }

                var links = document.querySelectorAll('link[rel="stylesheet"]');
                if (links.length === 0) {
                    // No stylesheets yet — poll until one appears
                    var poll = setInterval(function() {
                        var l = document.querySelectorAll('link[rel="stylesheet"]');
                        if (l.length > 0) {
                            clearInterval(poll);
                            waitForStylesheets(l);
                        }
                    }, 50);
                    setTimeout(function() { clearInterval(poll); reveal(); }, 1200);
                    return;
                }
                waitForStylesheets(links);

                function waitForStylesheets(links) {
                    var remaining = links.length;
                    function done() {
                        remaining--;
                        if (remaining <= 0) reveal();
                    }
                    for (var i = 0; i < links.length; i++) {
                        // Check if already loaded (sheet exists and is accessible)
                        var loaded = false;
                        try { loaded = links[i].sheet && links[i].sheet.cssRules && links[i].sheet.cssRules.length > 0; } catch(e) { loaded = false; }
                        if (loaded) {
                            done();
                        } else if (links[i].complete) {
                            done();
                        } else {
                            links[i].addEventListener('load', done);
                            links[i].addEventListener('error', done);
                        }
                    }
                    // Safety: reveal after 3s no matter what
                    setTimeout(reveal, 1200);
                }
            })();
        "#);
    });

    rsx! {
        // CRITICAL: Hide body until CSS + fonts load — prevents FOUC.
        // Dioxus mobile ignores index.html, generates minimal HTML with body visible.
        // This \u003cstyle\u003e in \u003chead\u003e hides body + sets themed background BEFORE any paint.
        document::Style { {FOUC_PREVENTION_CSS} }
        // Inject @font-face with Dioxus-resolved URLs — ensures fonts are bundled in APK
        document::Style { {font_face_css()} }
        document::Link { rel: "icon", r#type: "image/png", href: FAVICON.to_string() }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON.to_string() }
        document::Stylesheet { href: TAILWIND_CSS }
        // Wrapper div uses CSS variables for theme-aware background/text.
        div {
            class: "min-h-dvh bg-background text-foreground",
            style: "font-family:'Cabinet Grotesk',system-ui,-apple-system,sans-serif;",
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
fn ServerSettings() -> Element {
    rsx! { crate::auth::server_settings::ServerSettingsPage {} }
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
#[allow(non_snake_case)]
fn Accounts() -> Element {
    rsx! { SettingsAccounts {} }
}

#[allow(non_snake_case)]
fn Categories() -> Element {
    rsx! { SettingsCategories {} }
}

#[component]
fn Debts() -> Element {
    rsx! { crate::pages::DebtsPage {} }
}

#[component]
fn Reports() -> Element {
    rsx! { crate::pages::ReportsPage {} }
}

#[component]
fn Settings() -> Element {
    rsx! { crate::pages::SettingsProfilePage {} }
}

#[component]
fn SettingsProfile() -> Element {
    rsx! { crate::pages::SettingsProfilePage {} }
}

#[component]
fn SettingsPreferences() -> Element {
    rsx! { crate::pages::SettingsPreferencesPage {} }
}

#[component]
fn SettingsMoney() -> Element {
    rsx! { crate::pages::SettingsMoneyPage {} }
}

#[component]
fn SettingsNotifications() -> Element {
    rsx! { crate::pages::SettingsNotificationsPage {} }
}

#[component]
fn SettingsAboutPage() -> Element {
    rsx! { crate::pages::SettingsAboutPage {} }
}

#[component]
fn SettingsAccounts() -> Element {
    rsx! { crate::pages::AccountsPage {} }
}

#[component]
fn SettingsCategories() -> Element {
    rsx! { crate::pages::CategoriesPage {} }
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
