//! Recurrency on Native Shell.
//!
//! The migration keeps the product surface and design tokens of the Dioxus
//! app while replacing the renderer. Screens are ported one at a time; this
//! first slice owns the app frame and the dashboard's visual shell.

mod dashboard;
mod theme;
mod ui;

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
    view()
        .flex_1()
        .bg(move || theme().background)
        .font_family(theme::SANS)
        .text_color(move || theme().foreground)
        .child(ui::top_bar(route))
        .child(
            view()
                .flex_1()
                .w_full()
                .items_center()
                                .child(
                    view()
                        .w_full()
                        .max_w(px(1280.0))
                        .flex_1()
                        .px(px(40.0))
                        .py(px(32.0))
                        .child(match route.get() {
                            Route::Dashboard => dashboard::dashboard(),
                            _ => ui::pending_screen(route.get().label()),
                        }),
                ),
        )
}
