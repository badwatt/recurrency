//! Authenticated monthly plan: accounts, pending payments and the ledger.

use chrono::{Datelike, Local};
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use uuid::Uuid;

use crate::api;
use crate::auth::AuthSession;
use crate::dashboard_data::{
    pending_ratio, total_budget, total_pending, AccountBudget, CategoryBudget, DashboardData,
    UpcomingPayment,
};
use crate::format::{days_text, format_currency, month_label, short_date};
use crate::ui;
use crate::Route;

const PAGE_TITLE: &str = "This month's plan";
const PAGE_SUBTITLE: &str = "What to transfer into each account, and what is still left to pay.";

pub fn dashboard(auth: AuthSession, route: Signal<Route>, intent: Signal<Option<Route>>, compact: Signal<bool>) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let user_name = auth
        .user
        .name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "there".to_string());
    let base_currency = currency.clone();
    let data = resource(
        || Some(()),
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            let currency = base_currency.clone();
            async move { api::load_dashboard(server, token, currency).await }
        },
    );

    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(30.0))
        .child(header(&user_name, route, intent, compact))
        .child(
            when(loading, loading_card).otherwise(move || {
                let value = render_currency.clone();
                when(has_error, move || {
                    error_screen(&data.error().unwrap_or_default(), data)
                })
                .otherwise(move || body(data, value.clone(), route))
            }),
        )
        .into_element()
}

fn body(data: Resource<DashboardData, String>, currency: String, route: Signal<Route>) -> AnyElement {
    let today = Local::now().date_naive();
    let month = month_label(today);
    let days_left = crate::recurrence::days_in_month(today.year(), today.month())
        .saturating_sub(today.day());
    let hovered = signal(None::<Uuid>);
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(hero(data, &currency, &month, days_left))
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .gap(px(16.0))
                .items_start()
                .child(upcoming_card(data, &currency, route, today, hovered))
                .child(ledger_card(data, &currency, hovered)),
        )
        .into_element()
}

fn header(
    user_name: &str,
    route: Signal<Route>,
    intent: Signal<Option<Route>>,
    compact: Signal<bool>,
) -> impl IntoElement {
    let name = user_name.to_string();
    let mobile_name = name.clone();
    view().w_full().child(
        when(
            move || compact.get(),
            move || {
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(12.0))
                    .child(title_block(mobile_name.clone()))
                    .child(actions(route, intent))
                    .into_element()
            },
        )
        .otherwise(move || {
            view()
                .w_full()
                .flex_row()
                .items_end()
                .justify_between()
                .gap(px(24.0))
                .child(title_block(name.clone()))
                .child(actions(route, intent))
                .into_element()
        }),
    )
}

fn title_block(user_name: String) -> impl IntoElement {
    view()
        .flex_1()
        .min_w(px(0.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text(format!("Welcome back, {user_name}"))
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground)
                .line_height(px(20.0)),
        )
        .child(
            text(PAGE_TITLE)
                .text_size(px(36.0))
                .line_height(px(40.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(PAGE_SUBTITLE)
                .text_size(px(14.0))
                .line_height(px(20.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn actions(route: Signal<Route>, intent: Signal<Option<Route>>) -> impl IntoElement {
    view()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.0))
        .child(action("Add subscription", true, route, intent, Route::Subscriptions, asset!("icons/plus.svg")))
        .child(action("Add transaction", false, route, intent, Route::Transactions, asset!("icons/transactions.svg")))
        .child(action("Add debt", false, route, intent, Route::Debts, asset!("icons/plus.svg")))
}

fn action(
    label: &'static str,
    primary: bool,
    route: Signal<Route>,
    intent: Signal<Option<Route>>,
    target: Route,
    icon: ImageAsset,
) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(40.0))
        .px(px(16.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || if primary { theme().primary } else { theme().card })
        .active(|state| state.opacity(0.8))
        .flex_row()
        .gap(px(6.0))
        .on_press(move || {
            intent.set(Some(target));
            route.set(target);
        })
        .child(
            image(icon)
                .size(px(14.0))
                .tint_color(move || if primary { theme().primary_foreground } else { theme().foreground }),
        )
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || if primary {
                    theme().primary_foreground
                } else {
                    theme().foreground
                }),
        )
}

fn hero(
    data: Resource<DashboardData, String>,
    currency: &str,
    month: &str,
    days_left: u32,
) -> impl IntoElement {
    let currency = currency.to_string();
    let month = month.to_string();
    let budget = move || total_budget(&data.get().unwrap_or_default().accounts);
    let pending = move || total_pending(&data.get().unwrap_or_default().accounts);
    let ratio = move || pending_ratio(budget(), pending());
    let count = move || data.get().unwrap_or_default().accounts.len();
    let pending_count = move || {
        data.get()
            .unwrap_or_default()
            .accounts
            .iter()
            .map(|entry| entry.pending_count)
            .sum::<usize>()
    };
    let manual_count = move || {
        data.get()
            .unwrap_or_default()
            .accounts
            .iter()
            .map(|entry| entry.manual_count)
            .sum::<usize>()
    };
    let active = move || data.get().unwrap_or_default().active_subscriptions;
    let over_budget = move || pending() > budget();
    let currency_ratio = currency.clone();
    let currency_budget = currency.clone();
    let currency_pending = currency.clone();
    let ratio_label = move || {
        if budget() <= rust_decimal::Decimal::ZERO {
            "No budget this month.".to_string()
        } else if over_budget() {
            format!(
                "Over the plan by {}",
                format_currency(pending() - budget(), &currency_ratio)
            )
        } else {
            format!("{:.0}% of the budget pending", ratio())
        }
    };
    let pending_note = move || {
        let count = pending_count();
        if count == 0 {
            "All covered through today.".to_string()
        } else if days_left == 0 {
            format!("{} · last day of the month", ui::count_label(count, "payment", "payments"))
        } else {
            format!(
                "{} · {days_left} days left",
                ui::count_label(count, "payment", "payments")
            )
        }
    };

    view()
        .w_full()
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .overflow_hidden()
        .flex_row()
        .flex_wrap()
        .child(
            view()
                .flex_grow(1.15)
                .basis(px(400.0))
                .p(px(32.0))
                .child(
                    text(format!("PLAN FOR {month}"))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text("Transfer this month")
                        .text_size(px(14.0))
                        .text_color(move || theme().muted_foreground)
                        .mt(px(20.0)),
                )
                .child(
                    text(move || format_currency(budget(), &currency_budget))
                        .text_size(px(56.0))
                        .font_family(crate::theme::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Actual charges this month + manual payments.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground)
                        .mt(px(8.0)),
                )
                .child(
                    view()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(20.0))
                        .mt(px(22.0))
                        .child(stat(move || ui::count_label(count(), "account", "accounts")))
                        .child(stat(move || {
                            ui::count_label(active(), "active subscription", "active subscriptions")
                        }))
                        .child(stat(move || {
                            ui::count_label(manual_count(), "manual payment", "manual payments")
                        })),
                ),
        )
        .child(
            view()
                .flex_grow(0.85)
                .basis(px(300.0))
                .p(px(32.0))
                .border_l(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().muted.opacity(0.2))
                .child(
                    text("PENDING FROM TODAY")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(move || format_currency(pending(), &currency_pending))
                        .text_size(px(36.0))
                        .font_family(crate::theme::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground)
                        .mt(px(18.0)),
                )
                .child(
                    text(pending_note)
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground)
                        .mt(px(8.0)),
                )
                .child(
                    view()
                        .h(px(8.0))
                        .w_full()
                        .rounded(px(999.0))
                        .bg(move || theme().border)
                        .overflow_hidden()
                        .flex_row()
                        .mt(px(20.0))
                        .child(
                            view()
                                .h_full()
                                .flex_grow(ratio().max(0.01) as f32)
                                .bg(move || theme().primary),
                        )
                        .child(view().h_full().flex_grow((100.0 - ratio()).max(0.01) as f32)),
                )
                .child(
                    text(ratio_label)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground)
                        .mt(px(8.0)),
                ),
        )
}

fn stat(label: impl Fn() -> String + 'static) -> impl IntoElement {
    text(label)
        .text_size(px(12.0))
        .text_color(move || theme().muted_foreground)
}

fn upcoming_card(
    data: Resource<DashboardData, String>,
    currency: &str,
    route: Signal<Route>,
    today: chrono::NaiveDate,
    hovered: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let currency = currency.to_string();
    let total = move || {
        data.get()
            .unwrap_or_default()
            .upcoming
            .len()
    };
    view()
        .flex_grow(1.0)
        .basis(px(500.0))
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            view()
                .p(px(22.0))
                .pb(px(8.0))
                .flex_row()
                .items_start()
                .justify_between()
                .child(
                    view()
                        .flex_col()
                        .gap(px(2.0))
                        .child(section_title("UPCOMING PAYMENTS"))
                        .child(
                            text("Subscriptions and one-time payments.")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    button()
                        .items_center()
                        .justify_center()
                        .h(px(28.0))
                        .px(px(4.0))
                        .rounded(px(6.0))
                        .on_press(move || route.set(Route::Calendar))
                        .child(
                            text("View calendar")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().primary),
                        ),
                ),
        )
        .when(total() == 0, |card| {
            card.child(
                text("No upcoming payments or future transactions.")
                    .text_size(px(14.0))
                    .text_color(move || theme().muted_foreground)
                    .p(px(22.0))
                    .pt(px(8.0)),
            )
        })
        .when(total() > 0, |card| {
            card.child(
                view()
                    .px(px(22.0))
                    .pb(px(12.0))
                    .child(each(
                        move || {
                            let mut upcoming = data.get().unwrap_or_default().upcoming;
                            upcoming.truncate(5);
                            upcoming
                        },
                        |payment| payment.id,
                        move |payment, _| {
                            let payment = payment.get();
                            upcoming_row(payment, data, currency.clone(), today, hovered)
                        },
                    )),
            )
        })
}

fn upcoming_row(
    payment: UpcomingPayment,
    data: Resource<DashboardData, String>,
    currency: String,
    today: chrono::NaiveDate,
    hovered: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let converted = {
        let rates = data.get().unwrap_or_default().rates;
        rates.convert(payment.amount, &payment.currency)
    };
    let original = payment.currency != currency;
    let date = short_date(payment.date);
    let days = days_text(payment.date, today);
    let is_transaction = payment.is_transaction;
    let id = payment.id;
    let name = payment.name;
    let account = payment.account_name;
    let amount = format_currency(converted, &currency);
    let original = original.then(|| format_currency(payment.amount, &payment.currency));

    view()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .py(px(10.0))
        .gap(px(12.0))
        .rounded(px(8.0))
        .bg(move || if hovered.get() == Some(id) { theme().accent } else { theme().card })
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(Some(id)))
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(None))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .min_w(px(0.0))
                .flex_1()
                .child(
                    view()
                        .h(px(40.0))
                        .w(px(40.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().muted)
                        .child(
                            image(asset!("icons/card.svg"))
                            .size(px(17.0))
                            .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    text(format!("{account} · {date}"))
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                )
                                .when(is_transaction, |row| {
                                    row.child(
                                        text("ONE-TIME")
                                            .text_size(px(9.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(move || theme().muted_foreground)
                                            .px(px(4.0))
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .bg(move || theme().muted),
                                    )
                                }),
                        ),
                ),
        )
        .child(
            view()
                .flex_col()
                .items_end()
                .child(
                    text(amount)
                        .text_size(px(14.0))
                        .font_family(crate::theme::MONO)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .when(original.is_some(), |column| {
                    column.child(
                        text(original.unwrap_or_default())
                            .text_size(px(10.0))
                            .text_color(move || theme().muted_foreground),
                    )
                })
                .child(
                    text(days)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
}

fn ledger_card(
    data: Resource<DashboardData, String>,
    currency: &str,
    hovered: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let currency = currency.to_string();
    let currency_accounts = currency.clone();
    let currency_categories = currency.clone();
    let show_categories = signal(false);
    let account_count = move || data.get().unwrap_or_default().accounts.len();
    let category_count = move || data.get().unwrap_or_default().categories.len();
    view()
        .w(px(460.0))
        .flex_shrink(0.0)
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .overflow_hidden()
        .child(
            view()
                .p(px(22.0))
                .pb(px(12.0))
                .gap(px(8.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(section_title("TRANSFER BY ACCOUNT"))
                        .child(
                            view()
                                .flex_row()
                                .p(px(2.0))
                                .rounded(px(7.0))
                                .bg(move || theme().muted)
                                .child(tab("Accounts", !show_categories.get(), {
                                    let show_categories = show_categories;
                                    move || show_categories.set(false)
                                }))
                                .child(tab("Categories", show_categories.get(), {
                                    let show_categories = show_categories;
                                    move || show_categories.set(true)
                                })),
                        ),
                )
                .child(
                    text(move || {
                        if show_categories.get() {
                            "This month's charges grouped by category.".to_string()
                        } else {
                            "Actual charges this month and pending from today.".to_string()
                        }
                    })
                    .text_size(px(12.0))
                    .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .flex_row()
                .px(px(22.0))
                .py(px(8.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .child(column_header("ACCOUNT", 1.6))
                .child(column_header("BUDGET", 1.0))
                .child(column_header("PENDING", 1.0)),
        )
        .when(!show_categories.get(), move |card| {
            card.child(account_rows(data, &currency_accounts, hovered))
                .child(ledger_footer(
                    move || ui::count_label(account_count(), "account", "accounts"),
                    move || {
                        total_budget(
                            &data
                                .get()
                                .unwrap_or_default()
                                .accounts,
                        )
                    },
                    move || {
                        total_pending(
                            &data
                                .get()
                                .unwrap_or_default()
                                .accounts,
                        )
                    },
                    &currency_accounts,
                ))
        })
        .when(show_categories.get(), move |card| {
            card.child(category_rows(data, &currency_categories))
                .child(ledger_footer(
                    move || ui::count_label(category_count(), "category", "categories"),
                    move || {
                        data.get()
                            .unwrap_or_default()
                            .categories
                            .iter()
                            .map(|entry| entry.amount)
                            .sum()
                    },
                    move || {
                        data.get()
                            .unwrap_or_default()
                            .categories
                            .iter()
                            .map(|entry| entry.pending)
                            .sum()
                    },
                    &currency_categories,
                ))
        })
}

fn account_rows(
    data: Resource<DashboardData, String>,
    currency: &str,
    hovered: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let currency = currency.to_string();
    view()
        .flex_col()
        .child(each(
            move || data.get().unwrap_or_default().accounts,
            |entry| entry.account_id,
            move |entry, _| account_row(entry.get(), currency.clone(), hovered),
        ))
}

fn account_row(
    entry: AccountBudget,
    currency: String,
    hovered: Signal<Option<Uuid>>,
) -> impl IntoElement {
    let name = entry.name;
    let counts = format!(
        "{} subscriptions · {} manual",
        entry.subscription_count, entry.manual_count
    );
    let budget = format_currency(entry.budget, &currency);
    let pending = format_currency(entry.pending, &currency);
    let pending_count = entry.pending_count;
    let id = entry.account_id;

    view()
        .w_full()
        .flex_row()
        .items_center()
        .px(px(22.0))
        .py(px(12.0))
        .bg(move || if hovered.get() == Some(id) { theme().accent } else { theme().card })
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(Some(id)))
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(None))
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .child(
            view()
                .flex_grow(1.6)
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .min_w(px(0.0))
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().muted)
                        .child(
                            image(asset!("icons/card.svg"))
                                .size(px(16.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(counts)
                                .text_size(px(10.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(amount_value(budget, 1.0))
        .child(
            view()
                .flex_grow(1.0)
                .items_end()
                .child(amount_value(pending, 1.0))
                .child(
                    text(format!("{pending_count} payments"))
                        .text_size(px(10.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
}

fn category_rows(data: Resource<DashboardData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    view()
        .flex_col()
        .child(each(
            move || data.get().unwrap_or_default().categories,
            |entry| entry.category_id,
            move |entry, _| category_row(entry.get(), currency.clone()),
        ))
}

fn category_row(entry: CategoryBudget, currency: String) -> impl IntoElement {
    let name = entry.name;
    let count = format!("{} charges", entry.charge_count);
    let amount = format_currency(entry.amount, &currency);
    let pending = format_currency(entry.pending, &currency);
    view()
        .w_full()
        .flex_row()
        .items_center()
        .px(px(22.0))
        .py(px(12.0))
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .child(
            view()
                .flex_grow(1.6)
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .min_w(px(0.0))
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().muted)
                        .child(
                            image(asset!("icons/card.svg"))
                                .size(px(16.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(count)
                                .text_size(px(10.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(amount_value(amount, 1.0))
        .child(amount_value(pending, 1.0))
}

fn ledger_footer(
    label: impl Fn() -> String + 'static,
    budget: impl Fn() -> rust_decimal::Decimal + 'static,
    pending: impl Fn() -> rust_decimal::Decimal + 'static,
    currency: &str,
) -> impl IntoElement {
    let currency = currency.to_string();
    let pending_currency = currency.clone();
    view()
        .w_full()
        .flex_row()
        .items_center()
        .px(px(22.0))
        .py(px(14.0))
        .bg(move || theme().muted)
        .child(
            view()
                .flex_grow(1.6)
                .flex_col()
                .child(
                    text("MONTH TOTAL")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(label)
                        .text_size(px(10.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(amount_cell(
            move || format_currency(budget(), &currency),
            1.0,
        ))
        .child(amount_cell(
            move || format_currency(pending(), &pending_currency),
            1.0,
        ))
}

fn amount_value(amount: String, grow: f32) -> impl IntoElement {
    view()
        .flex_grow(grow)
        .items_end()
        .child(
            text(amount)
                .text_size(px(13.0))
                .font_family(crate::theme::MONO)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
}

fn amount_cell(
    amount: impl Fn() -> String + 'static,
    grow: f32,
) -> impl IntoElement {
    view()
        .flex_grow(grow)
        .items_end()
        .child(
            text(amount)
                .text_size(px(13.0))
                .font_family(crate::theme::MONO)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
}

fn tab(label: &'static str, active: bool, on_press: impl Fn() + 'static) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(5.0))
        .bg(move || if active { theme().card } else { theme().muted })
        .on_press(on_press)
        .child(
            text(label)
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || if active {
                    theme().foreground
                } else {
                    theme().muted_foreground
                }),
        )
}

fn column_header(label: &'static str, grow: f32) -> impl IntoElement {
    view()
        .flex_grow(grow)
        .items_end()
        .child(
            text(label)
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().muted_foreground),
        )
}

fn section_title(label: &'static str) -> impl IntoElement {
    text(label)
        .text_size(px(12.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(move || theme().muted_foreground)
}

fn loading_card() -> AnyElement {
    ui::surface()
        .p(px(32.0))
        .child(
            text("Loading your plan…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_screen(message: &str, data: Resource<DashboardData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
                .p(px(24.0))
                .gap(px(12.0))
                .child(
                    text("The dashboard could not be loaded.")
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(message)
                        .text_size(px(13.0))
                        .text_color(move || theme().destructive),
                )
                .child(
                    button()
                        .items_center()
                        .justify_center()
                        .h(px(34.0))
                        .px(px(14.0))
                        .rounded(px(7.0))
                        .bg(move || theme().primary)
                        .on_press(move || data.refetch())
                        .child(
                            text("Try again")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().primary_foreground),
                        ),
                )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_labels_use_the_singular_for_one() {
        assert_eq!(ui::count_label(1, "account", "accounts"), "1 account");
        assert_eq!(ui::count_label(2, "account", "accounts"), "2 accounts");
        assert_eq!(ui::count_label(0, "payment", "payments"), "0 payments");
    }
}
