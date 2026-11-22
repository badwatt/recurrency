//! Recurrency on Native Shell.
//!
//! The migration keeps the product surface and design tokens of the Dioxus
//! app while replacing the renderer. Screens are ported one at a time; this
//! first slice owns the app frame and the dashboard's visual shell.

mod api;
mod auth;
mod calendar;
pub mod components;
mod dashboard;
mod debts;
mod dashboard_data;
mod format;
mod login;
mod models;
mod recurrence;
mod reports;
mod settings;
mod subscriptions;
mod theme;
mod transactions;
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
    crate::theme::apply(&crate::theme::saved());
    ui::init_privacy();
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
        .child(image(LOGO).size(px(36.0)).tint_color(move || theme().primary))
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
    let sync_server = auth::stored_server();
    let sync_token = auth.token.clone();
    spawn(async move {
        if let Ok(preferences) = api::load_preferences(sync_server, sync_token).await {
            if preferences.theme_is_set && theme::theme_by_id(&preferences.theme).is_some() {
                theme::apply(&preferences.theme);
                theme::save(&preferences.theme);
            }
        }
    });
    let auth_signal = signal(auth);
    let compact = signal(false);
    let modal = ui::ModalHost::new();
    let create_intent = signal(None::<Route>);
    view()
        .relative()
        .flex_1()
        .on_layout(move |bounds: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
            compact.set(bounds.size.width.0 < 900.0);
        })
        .bg(move || theme().background)
        .font_family(theme::SANS)
        .text_color(move || theme().foreground)
        .child(ui::top_bar(route, auth_signal.get(), compact))
        .child(
            scroll().flex_1().child(
                view()
                    .w_full()
                    .items_center()
                    .child(
                        view()
                            .w_full()
                            .key(if ui::privacy_active() { "private" } else { "public" })
                            .max_w(px(1280.0))
                            .px(move || {
                                if compact.get() { px(16.0) } else if route.get() == Route::Dashboard { px(28.0) } else { px(24.0) }
                            })
                            .py(move || {
                                if compact.get() { px(16.0) } else if route.get() == Route::Dashboard { px(32.0) } else { px(24.0) }
                            })
                            .child(
                                when(
                                    move || route.get() == Route::Dashboard,
                                    move || dashboard::dashboard(auth_signal.get(), route, create_intent, compact),
                                )
                                .otherwise(move || {
                                    when(
                                        move || route.get() == Route::Subscriptions,
                                        move || subscriptions::subscriptions(auth_signal.get(), modal, create_intent),
                                    )
                                    .otherwise(move || {
                                        when(
                                            move || route.get() == Route::Transactions,
                                            move || transactions::transactions(auth_signal.get(), modal, create_intent),
                                        )
                                        .otherwise(move || {
                                            when(
                                                move || route.get() == Route::Debts,
                                                move || debts::debts(auth_signal.get(), modal, create_intent),
                                            )
                                            .otherwise(move || {
                                                when(
                                                    move || route.get() == Route::Calendar,
                                                    move || calendar::calendar(auth_signal.get(), compact),
                                                )
                                                .otherwise(move || {
                                                    when(
                                                        move || route.get() == Route::Reports,
                                                        move || reports::reports(auth_signal.get()),
                                                    )
                                                    .otherwise(move || {
                                                        when(
                                                            move || route.get() == Route::Settings,
                                                            move || settings::settings(auth_signal.get(), session),
                                                        )
                                                        .otherwise(move || ui::pending_screen(route.get().label()))
                                                    })
                                                })
                                            })
                                        })
                                    })
                                }),
                            ),
                    ),
            ),
        )
        .child(
            when(move || compact.get(), move || ui::bottom_bar(route))
                .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(move || modal.active(), move || ui::modal_overlay(modal))
                .otherwise(|| view().h(px(0.0))),
        )
        .into_element()
}
