//! Recurrency on Native Shell.
//!
//! The migration keeps the product surface and design tokens of the Dioxus
//! app while replacing the renderer. Screens are ported one at a time; this
//! first slice owns the app frame and the dashboard's visual shell.

mod ai_chat;
mod api;
mod auth;
mod calendar;
pub mod components;
mod currencies;
mod dashboard;
mod dashboard_data;
mod debts;
mod delivery;
mod folder_sync;
mod format;
mod local;
mod login;
mod mic;
mod models;
mod notifications;
mod recurrence;
mod reports;
mod settings;
mod subscriptions;
mod theme;
mod transactions;
mod ui;
mod update_banner;
mod updater;
mod view_state;
mod voice;
#[cfg(target_arch = "wasm32")]
mod web_files;

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

    pub fn key(self) -> &'static str {
        match self {
            Route::Dashboard => "dashboard",
            Route::Subscriptions => "subscriptions",
            Route::Transactions => "transactions",
            Route::Debts => "debts",
            Route::Calendar => "calendar",
            Route::Reports => "reports",
            Route::Settings => "settings",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Some(match value.trim().trim_start_matches('#') {
            "dashboard" => Route::Dashboard,
            "subscriptions" => Route::Subscriptions,
            "transactions" => Route::Transactions,
            "debts" => Route::Debts,
            "calendar" => Route::Calendar,
            "reports" => Route::Reports,
            "settings" => Route::Settings,
            _ => return None,
        })
    }

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
    #[cfg(target_arch = "wasm32")]
    crate::web_files::install_layout_css();
    #[cfg(target_arch = "wasm32")]
    crate::web_files::install_search_escape();
    #[cfg(target_arch = "wasm32")]
    crate::web_files::install_inline_select_dismiss();
    crate::theme::apply(&crate::theme::saved());
    crate::format::init_date_format();
    ui::init_privacy();
    ui::init_logo_modal();
    let route = signal(ui::stored_route());
    ui::init_route(route);
    effect(move || ui::persist_route(route.get()));
    let stored = auth::has_stored_session();
    let session = signal(if stored {
        Session::Loading
    } else {
        Session::SignedOut
    });
    if stored {
        spawn(async move { session.set(auth::restore_session().await) });
    }

    when(
        move || matches!(session.get(), Session::SignedIn(_)),
        move || app_shell(route, session),
    )
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
        .child(
            image(LOGO)
                .size(px(36.0))
                .tint_color(move || theme().primary),
        )
        .child(
            text("Recurrency")
                .text_size(px(19.0))
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
            format::set_date_format(&preferences.date_format);
            format::set_first_day(&preferences.first_day_of_week);
        }
    });
    let chat_auth = auth.clone();
    let notification_auth = chat_auth.clone();
    let creator_auth = auth.clone();
    let auth_signal = signal(auth);
    let compact = signal(false);
    ui::init_compact(compact);
    ui::init_selects();
    let modal = ui::ModalHost::new();
    ui::init_subscription_creator({
        let auth = creator_auth.clone();
        move |date| crate::subscriptions::open_create_dialog(auth.clone(), modal, date)
    });
    ui::init_transaction_creator({
        let auth = creator_auth.clone();
        move |date| crate::transactions::open_create_dialog(auth.clone(), modal, date)
    });
    ui::init_debt_creator({
        let auth = creator_auth;
        move |date| crate::debts::open_create_dialog(auth.clone(), modal, date)
    });
    let create_intent = signal(None::<Route>);
    let data_key = signal(0_u64);
    effect(move || data_key.set(ui::data_revision()));
    let fab_open = signal(false);
    let page_scroll = element_ref();
    effect(move || {
        let _ = route.get();
        page_scroll.scroll_to_offset(px(0.0), false);
    });
    view()
        .relative()
        .flex_1()
        .on_layout(
            move |bounds: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
                compact.set(bounds.size.width.0 < 900.0);
            },
        )
        .bg(move || theme().background)
        .font_family(theme::SANS)
        .text_color(move || theme().foreground)
        .child(ui::top_bar(route, auth_signal.get(), compact))
        .child(update_banner::update_banner())
        .child(
            scroll()
                .flex_1()
                .pr(px(10.0))
                .element_ref(&page_scroll)
                .child(
                    view().w_full().items_center().child(
                        view()
                            .w_full()
                            .key(format!(
                                "{}-{}-{}",
                                if compact.get() { "compact" } else { "wide" },
                                if ui::privacy_active() {
                                    "private"
                                } else {
                                    "public"
                                },
                                data_key.get()
                            ))
                            .max_w(px(1242.0))
                            .pl(move || {
                                if compact.get() {
                                    px(24.0)
                                } else if route.get() == Route::Dashboard {
                                    px(13.0)
                                } else {
                                    px(5.0)
                                }
                            })
                            .pr(move || {
                                if compact.get() {
                                    px(24.0)
                                } else if route.get() == Route::Dashboard {
                                    px(13.0)
                                } else {
                                    px(5.0)
                                }
                            })
                            .py(move || {
                                if compact.get() {
                                    px(16.0)
                                } else if route.get() == Route::Dashboard {
                                    px(31.0)
                                } else if route.get() == Route::Settings {
                                    px(24.0)
                                } else {
                                    px(29.0)
                                }
                            })
                            .child(
                                when(
                                    move || route.get() == Route::Dashboard,
                                    move || {
                                        dashboard::dashboard(
                                            auth_signal.get(),
                                            route,
                                            create_intent,
                                            compact,
                                            modal,
                                        )
                                    },
                                )
                                .otherwise(move || {
                                    when(
                                        move || route.get() == Route::Subscriptions,
                                        move || {
                                            subscriptions::subscriptions(
                                                auth_signal.get(),
                                                modal,
                                                create_intent,
                                                compact,
                                            )
                                        },
                                    )
                                    .otherwise(move || {
                                        when(
                                            move || route.get() == Route::Transactions,
                                            move || {
                                                transactions::transactions(
                                                    auth_signal.get(),
                                                    modal,
                                                    create_intent,
                                                    compact,
                                                )
                                            },
                                        )
                                        .otherwise(
                                            move || {
                                                when(
                                                    move || route.get() == Route::Debts,
                                                    move || {
                                                        debts::debts(
                                                            auth_signal.get(),
                                                            modal,
                                                            create_intent,
                                                            compact,
                                                        )
                                                    },
                                                )
                                                .otherwise(move || {
                                                    when(
                                                        move || route.get() == Route::Calendar,
                                                        move || {
                                                            calendar::calendar(
                                                                auth_signal.get(),
                                                                compact,
                                                            )
                                                        },
                                                    )
                                                    .otherwise(move || {
                                                        when(
                                                            move || route.get() == Route::Reports,
                                                            move || {
                                                                reports::reports(
                                                                    auth_signal.get(),
                                                                    compact,
                                                                )
                                                            },
                                                        )
                                                        .otherwise(move || {
                                                            when(
                                                                move || {
                                                                    route.get() == Route::Settings
                                                                },
                                                                move || {
                                                                    settings::settings(
                                                                        auth_signal.get(),
                                                                        session,
                                                                    )
                                                                },
                                                            )
                                                            .otherwise(move || {
                                                                ui::pending_screen(
                                                                    route.get().label(),
                                                                )
                                                            })
                                                        })
                                                    })
                                                })
                                            },
                                        )
                                    })
                                }),
                            ),
                    ),
                ),
        )
        .child(
            when(
                move || {
                    compact.get()
                        && matches!(
                            route.get(),
                            Route::Dashboard
                                | Route::Subscriptions
                                | Route::Transactions
                                | Route::Debts
                        )
                },
                move || {
                    let ai_route = route;
                    let ai_auth = auth_signal;
                    crate::components::fab_menu::fab_menu(
                        fab_open,
                        vec![
                            crate::components::fab_menu::FabAction::new(
                                "AI assistant",
                                asset!("icons/sparkles.svg"),
                                move || ui::open_ai(ai_route, ai_auth.get()),
                            ),
                            crate::components::fab_menu::FabAction::new(
                                "New subscription",
                                asset!("icons/receipt.svg"),
                                move || ui::create_subscription(None),
                            ),
                            crate::components::fab_menu::FabAction::new(
                                "New transaction",
                                asset!("icons/wallet.svg"),
                                move || ui::create_transaction(None),
                            ),
                            crate::components::fab_menu::FabAction::new(
                                "New debt",
                                asset!("icons/notes.svg"),
                                move || ui::create_debt(None),
                            ),
                        ],
                    )
                    .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(move || compact.get(), move || ui::bottom_bar(route))
                .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(move || modal.active(), move || ui::modal_overlay(modal))
                .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(
                move || ui::ai_chat_open(),
                move || ai_chat::ai_chat(chat_auth.clone()),
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(
                move || ui::notifications_open(),
                move || {
                    view()
                        .absolute()
                        .inset_0()
                        .z_index(100)
                        .items_center()
                        .justify_center()
                        .p(px(16.0))
                        .bg(rgba(0x000000cc))
                        .child(notifications::notifications_modal(
                            notification_auth.clone(),
                        ))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(
                move || ui::logo_modal_active(),
                move || {
                    view()
                        .absolute()
                        .inset_0()
                        .z_index(110)
                        .child(ui::logo_modal_overlay())
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(ui::menu_overlay())
        .child(crate::components::toast::toast_overlay())
        .into_element()
}
