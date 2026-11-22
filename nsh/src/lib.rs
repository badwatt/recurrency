//! Recurrency on Native Shell.
//!
//! The migration keeps the product surface and design tokens of the Dioxus
//! app while replacing the renderer. Screens are ported one at a time; this
//! first slice owns the app frame and the dashboard's visual shell.

mod api;
mod auth;
mod dashboard;
mod dashboard_data;
mod format;
mod login;
mod models;
mod recurrence;
mod subscriptions;
mod theme;
mod ui;

use auth::Session;
use nsh_base::theme;
use nsh_core::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Dashboard,
    Subscriptions,
    Transactions,
    Debts,
    Calendar,
    Reports,
    Settings,
}

impl Route {
    pub const NAV: [Route; 6] = [
        Route::Dashboard,
        Route::Subscriptions,
        Route::Transactions,
        Route::Debts,
        Route::Calendar,
        Route::Reports,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Route::Dashboard => "Dashboard",
            Route::Subscriptions => "Subscriptions",
            Route::Transactions => "Transactions",
            Route::Debts => "Debts",
            Route::Calendar => "Calendar",
            Route::Reports => "Reports",
            Route::Settings => "Settings",
        }
    }

    pub(crate) fn icon(self) -> ImageAsset {
        match self {
            Route::Dashboard => asset!("icons/dashboard.svg"),
            Route::Subscriptions => asset!("icons/subscriptions.svg"),
            Route::Transactions => asset!("icons/transactions.svg"),
            Route::Debts => asset!("icons/debts.svg"),
            Route::Calendar => asset!("icons/calendar.svg"),
            Route::Reports => asset!("icons/reports.svg"),
            Route::Settings => asset!("icons/reports.svg"),
        }
    }
}

const LOGO: ImageAsset = asset!("icons/clock.svg");

#[native_module("recurrency/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>().assets(assets!());
}

/// Recurrency root component.
#[component]
pub fn App() -> impl IntoElement {
    let route = signal(Route::Dashboard);
    let stored = auth::has_stored_session();
    let session = signal(if stored { Session::Loading } else { Session::SignedOut });
    if stored {
        spawn(async move { session.set(auth::restore_session().await) });
    }

    when(move || matches!(session.get(), Session::SignedIn(_)), move || {
        app_shell(route, session)
    })
    .otherwise(move || match session.get() {
        Session::Loading => loading_screen(),
        _ => login::login_screen(session).into_element(),
    })
    .into_element()
}

fn loading_screen() -> AnyElement {
    view()
        .flex_1()
        .w_full()
        .items_center()
        .justify_center()
        .bg(move || theme().background)
        .gap(px(12.0))
        .child(image(LOGO).size(px(36.0)))
        .child(
            text("Recurrency")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .into_element()
}

fn app_shell(route: Signal<Route>, session: Signal<Session>) -> AnyElement {
    let auth = match session.get() {
        Session::SignedIn(auth) => auth,
        _ => return loading_screen(),
    };
    view()
        .flex_1()
        .bg(move || theme().background)
        .font_family(theme::SANS)
        .text_color(move || theme().foreground)
        .child(ui::top_bar(route))
        .child(
            scroll().flex_1().child(
                view()
                    .w_full()
                    .items_center()
                    .child(
                        view()
                            .w_full()
                            .max_w(px(1280.0))
                            .px(px(40.0))
                            .py(px(32.0))
                            .child(
                                when(
                                    move || route.get() == Route::Dashboard,
                                    {
                                        let auth = auth.clone();
                                        move || dashboard::dashboard(auth.clone(), route)
                                    },
                                )
                                .otherwise(move || {
                                    let subscriptions_auth = auth.clone();
                                    when(
                                        move || route.get() == Route::Subscriptions,
                                        move || subscriptions::subscriptions(subscriptions_auth.clone()),
                                    )
                                    .otherwise(move || ui::pending_screen(route.get().label()))
                                }),
                            ),
                    ),
            ),
        )
        .into_element()
}
