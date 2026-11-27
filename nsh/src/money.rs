//! Unified money page: subscriptions, budgets, one-off payments and debts.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api;
use crate::auth::AuthSession;
use crate::components::card::card;
use crate::components::converted_amount::converted_amount;
use crate::components::empty_state::empty_state;
use crate::components::page_header::page_header;
use crate::components::search_input::search_input;
use crate::ui;
use crate::format::format_date;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Subscription,
    Budget,
    Transaction,
    Debt,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Subscription => "Subscription",
            Self::Budget => "Budget",
            Self::Transaction => "One-off payment",
            Self::Debt => "Debt",
        }
    }

}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Subscription,
    Budget,
    Transaction,
    Debt,
}

impl Filter {
    fn accepts(self, kind: Kind) -> bool {
        match self {
            Self::All => true,
            Self::Subscription => kind == Kind::Subscription,
            Self::Budget => kind == Kind::Budget,
            Self::Transaction => kind == Kind::Transaction,
            Self::Debt => kind == Kind::Debt,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Subscription => "Subscriptions",
            Self::Budget => "Budgets",
            Self::Transaction => "One-off",
            Self::Debt => "Debts",
        }
    }
}

#[derive(Clone)]
struct Item {
    id: Uuid,
    kind: Kind,
    name: String,
    amount: Decimal,
    currency: String,
    logo: Option<String>,
    account: String,
    category: String,
    detail: String,
    progress: Option<f32>,
    updated: i64,
}

pub fn money(auth: AuthSession, modal: ui::ModalHost, compact: Signal<bool>) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let subscriptions = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_subscriptions(server, token, currency).await }
        }
    });
    let transactions = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_transactions(server, token, currency).await }
        }
    });
    let debts = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_debts(server, token, currency).await }
        }
    });
    let filter = signal(Filter::All);
    let search = signal(String::new());
    let list_view = signal(false);
    let loading = move || {
        (subscriptions.loading() && subscriptions.get().is_none())
            || (transactions.loading() && transactions.get().is_none())
            || (debts.loading() && debts.get().is_none())
    };
    let error = move || {
        subscriptions
            .error()
            .or_else(|| transactions.error())
            .or_else(|| debts.error())
    };
    let items = move || {
        build_items(
            subscriptions,
            transactions,
            debts,
            search.get(),
            filter.get(),
        )
    };
    let open_add = {
        let auth = std::rc::Rc::new(auth.clone());
        move || {
            let auth = auth.clone();
            modal.open(move || add_menu(auth.clone(), modal));
        }
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(22.0))
        .child(page_header(
            "Money",
            "Subscriptions, budgets, one-off payments and debts in one place.",
            view()
                .child(crate::components::button::button(
                    crate::components::button::ButtonProps::new("Add")
                        .icon(asset!("icons/plus.svg"))
                        .on_press(open_add.clone()),
                ))
                .into_element(),
            compact,
            false,
            0.0,
        ))
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(search_input(search, "Search money"))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(crate::components::list_toolbar::view_toggle(
                            asset!("icons/grid.svg"),
                            "Grid view",
                            move || !list_view.get(),
                            move || list_view.set(false),
                        ))
                        .child(crate::components::list_toolbar::view_toggle(
                            asset!("icons/list.svg"),
                            "List view",
                            move || list_view.get(),
                            move || list_view.set(true),
                        )),
                ),
        )
        .child(chips(filter, items.clone()))
        .child(
            when(loading, || {
                card()
                    .p(px(24.0))
                    .child(text("Loading money…"))
                    .into_element()
            })
            .otherwise(move || {
                let rest_currency = currency.clone();
                when(move || error().is_some(), move || {
                    card()
                        .p(px(24.0))
                        .child(text(error().unwrap_or_default()))
                        .into_element()
                })
                .otherwise(move || {
                    let branch_currency = rest_currency.clone();
                    when(move || items().is_empty(), move || {
                        empty_state(
                            asset!("icons/wallet.svg"),
                            "No money items yet",
                            "Add a subscription, budget, one-off payment or debt.",
                            crate::components::button::button(
                                crate::components::button::ButtonProps::new("Add")
                                    .icon(asset!("icons/plus.svg"))
                                    .on_press(move || crate::ui::create_subscription(None)),
                            )
                            .into_element(),
                        )
                        .into_element()
                    })
                    .otherwise(move || {
                        let base_rates = subscriptions
                            .get()
                            .map(|data| data.rates)
                            .or_else(|| transactions.get().map(|data| data.rates))
                            .or_else(|| debts.get().map(|data| data.rates))
                            .unwrap_or_default();
                        let base_currency = branch_currency.clone();
                        when(list_view.get(), {
                            let rates = base_rates.clone();
                            let currency = base_currency.clone();
                            move || {
                                view()
                                    .w_full()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .children(
                                        items()
                                            .into_iter()
                                            .map(|item| list_row(item, &currency, &rates)),
                                    )
                                    .into_element()
                            }
                        })
                        .otherwise(move || {
                            view()
                                .w_full()
                                .grid()
                                .grid_template_columns(SharedString::from(if compact.get() {
                                    "repeat(1, minmax(0, 1fr))"
                                } else {
                                    "repeat(3, minmax(0, 1fr))"
                                }))
                                .gap(px(16.0))
                                .children(
                                    items()
                                        .into_iter()
                                        .map(|item| grid_card(item, &base_currency, &base_rates)),
                                )
                                .into_element()
                        })
                        .into_element()
                    })
                    .into_element()
                })
            }),
        )
        .into_element()
}

fn chips(filter: Signal<Filter>, items: impl Fn() -> Vec<Item> + Copy + 'static) -> AnyElement {
    let counts = move || {
        let all = items();
        [
            (Filter::All, all.len()),
            (
                Filter::Subscription,
                all.iter().filter(|i| i.kind == Kind::Subscription).count(),
            ),
            (
                Filter::Budget,
                all.iter().filter(|i| i.kind == Kind::Budget).count(),
            ),
            (
                Filter::Transaction,
                all.iter().filter(|i| i.kind == Kind::Transaction).count(),
            ),
            (
                Filter::Debt,
                all.iter().filter(|i| i.kind == Kind::Debt).count(),
            ),
        ]
    };
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .gap(px(6.0))
        .child(each(
            move || counts(),
            |(value, count)| format!("{}-{}", value.label(), count),
            move |item, _| {
                let (value, count) = item.get();
                let active = move || filter.get() == value;
                button()
                    .cursor_pointer()
                    .h(px(32.0))
                    .px(px(12.0))
                    .items_center()
                    .justify_center()
                    .rounded(px(999.0))
                    .border(px(1.0))
                    .border_color(move || {
                        if active() {
                            theme().primary
                        } else {
                            theme().border
                        }
                    })
                    .bg(move || {
                        if active() {
                            theme().primary.opacity(0.12)
                        } else {
                            theme().card
                        }
                    })
                    .on_press(move || filter.set(value))
                    .child(
                        text(format!("{} · {}", value.label(), count))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || {
                                if active() {
                                    theme().primary
                                } else {
                                    theme().muted_foreground
                                }
                            }),
                    )
                    .into_element()
            },
        ))
        .into_element()
}

fn build_items(
    subscriptions: Resource<api::SubscriptionsData, String>,
    transactions: Resource<api::TransactionsData, String>,
    debts: Resource<api::DebtsData, String>,
    search: String,
    filter: Filter,
) -> Vec<Item> {
    let needle = search.trim().to_lowercase();
    let mut items = Vec::new();
    if let Some(data) = subscriptions.get() {
        for sub in data.subscriptions {
            let kind = if sub.kind == "budget" {
                Kind::Budget
            } else {
                Kind::Subscription
            };
            let account = data
                .accounts
                .iter()
                .find(|a| a.id == sub.account_id)
                .map_or_else(String::new, |a| a.name.clone());
            let category = data
                .categories
                .iter()
                .find(|c| c.id == sub.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            let detail = sub.next_billing_date.map_or_else(
                || "No next date".to_string(),
                |date| format!("Next: {}", format_date(date)),
            );
            items.push(Item {
                id: sub.id,
                kind,
                name: sub.name,
                amount: sub.amount,
                currency: sub.currency,
                logo: sub.logo_url,
                account,
                category,
                detail,
                progress: None,
                updated: sub.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    if let Some(data) = transactions.get() {
        for tx in data.transactions {
            let account = data
                .accounts
                .iter()
                .find(|a| a.id == tx.account_id)
                .map_or_else(String::new, |a| a.name.clone());
            let category = data
                .categories
                .iter()
                .find(|c| c.id == tx.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            items.push(Item {
                id: tx.id,
                kind: Kind::Transaction,
                name: tx.name,
                amount: tx.amount,
                currency: tx.currency,
                logo: tx.logo_url,
                account,
                category,
                detail: format_date(tx.date),
                progress: None,
                updated: tx.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    if let Some(data) = debts.get() {
        for debt in data.debts {
            let account = data
                .accounts
                .iter()
                .find(|a| a.id == debt.account_id)
                .map_or_else(String::new, |a| a.name.clone());
            let category = data
                .categories
                .iter()
                .find(|c| c.id == debt.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            let paid = debt.total_amount - debt.remaining_amount;
            let ratio = if debt.total_amount > Decimal::ZERO {
                (paid / debt.total_amount)
                    .to_string()
                    .parse::<f32>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            items.push(Item {
                id: debt.id,
                kind: Kind::Debt,
                name: debt.name,
                amount: debt.remaining_amount,
                currency: debt.currency,
                logo: debt.logo_url,
                account,
                category,
                detail: debt.next_payment_date.map_or_else(
                    || "No next payment".to_string(),
                    |date| format!("Next: {}", format_date(date)),
                ),
                progress: Some(ratio.clamp(0.0, 1.0)),
                updated: debt.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    items.retain(|item| {
        filter.accepts(item.kind)
            && (needle.is_empty()
                || item.name.to_lowercase().contains(&needle)
                || item.account.to_lowercase().contains(&needle)
                || item.category.to_lowercase().contains(&needle))
    });
    items.sort_by_key(|item| std::cmp::Reverse(item.updated));
    items
}

fn kind_icon(kind: Kind) -> ImageAsset {
    match kind {
        Kind::Subscription | Kind::Budget => asset!("icons/subscriptions.svg"),
        Kind::Transaction => asset!("icons/transactions.svg"),
        Kind::Debt => asset!("icons/debts.svg"),
    }
}

fn type_badge(kind: Kind) -> AnyElement {
    let color = match kind {
        Kind::Subscription => theme().primary,
        Kind::Budget => theme().accent_foreground,
        Kind::Transaction => theme().muted_foreground,
        Kind::Debt => theme().destructive,
    };
    view()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(999.0))
        .bg(move || theme().muted.opacity(0.6))
        .child(
            text(kind.label())
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || color),
        )
        .into_element()
}

fn grid_card(item: Item, currency: &str, rates: &crate::models::Rates) -> AnyElement {
    let logo = item.logo.clone();
    let name = item.name.clone();
    let detail = item.detail.clone();
    let account = item.account.clone();
    let kind = item.kind;
    let progress = item.progress;
    card()
        .w_full()
        .p(px(18.0))
        .gap(px(12.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(crate::components::logo_avatar::logo_avatar(
                    logo,
                    kind_icon(kind),
                    38.0,
                    false,
                ))
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .flex_grow(1.0)
                        .child(
                            text(name)
                                .number_of_lines(1)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(detail)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(converted_amount(
                    item.amount,
                    &item.currency,
                    currency,
                    rates,
                    15.0,
                    FontWeight::BOLD,
                    true,
                )),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(type_badge(kind))
                .child(
                    text(if account.is_empty() {
                        item.category.clone()
                    } else {
                        account
                    })
                    .text_size(px(11.0))
                    .text_color(move || theme().muted_foreground),
                ),
        )
        .when_some(progress, |card, value| {
            card.child(
                view()
                    .w_full()
                    .h(px(6.0))
                    .rounded(px(999.0))
                    .bg(move || theme().muted)
                    .child(
                        view()
                            .h_full()
                            .w(nsh_core::geometry::Length::from(
                                nsh_core::geometry::Percentage(value),
                            ))
                            .rounded(px(999.0))
                            .bg(move || theme().primary),
                    ),
            )
        })
        .into_element()
}

fn list_row(item: Item, currency: &str, rates: &crate::models::Rates) -> AnyElement {
    let logo = item.logo.clone();
    let name = item.name.clone();
    let detail = item.detail.clone();
    let kind = item.kind;
    let progress = item.progress;
    view()
        .w_full()
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(14.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(crate::components::logo_avatar::logo_avatar(
                    logo,
                    kind_icon(kind),
                    36.0,
                    false,
                ))
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .flex_grow(1.0)
                        .child(
                            text(name)
                                .number_of_lines(1)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(detail)
                                .number_of_lines(1)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(type_badge(kind)),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .mt(px(10.0))
                .pt(px(8.0))
                .border_t(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .child(converted_amount(
                    item.amount,
                    &item.currency,
                    currency,
                    rates,
                    14.0,
                    FontWeight::SEMIBOLD,
                    true,
                ))
                .when_some(progress, |row, value| {
                    row.child(
                        view()
                            .w(px(80.0))
                            .h(px(6.0))
                            .rounded(px(999.0))
                            .bg(move || theme().muted)
                            .child(
                                view()
                                    .h_full()
                                    .w(nsh_core::geometry::Length::from(
                                        nsh_core::geometry::Percentage(value),
                                    ))
                                    .rounded(px(999.0))
                                    .bg(move || theme().primary),
                            ),
                    )
                }),
        )
        .into_element()
}

fn add_menu(auth: std::rc::Rc<AuthSession>, modal: ui::ModalHost) -> AnyElement {
    let close = move || modal.close();
    let open = move |kind: Filter| {
        modal.close();
        match kind {
            Filter::Subscription => crate::ui::create_subscription(None),
            Filter::Budget => crate::subscriptions::open_create_dialog_kind(
                (*auth).clone(),
                modal,
                None,
                "budget",
            ),
            Filter::Transaction => crate::ui::create_transaction(None),
            Filter::Debt => crate::ui::create_debt(None),
            Filter::All => {}
        }
    };
    let row = |kind: Filter,
               icon: ImageAsset,
               description: &'static str,
               open: &std::rc::Rc<dyn Fn(Filter)>| {
        let open = open.clone();
        view()
            .w_full()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .p(px(12.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .border(px(1.0))
            .border_color(move || theme().border)
            .bg(move || theme().card)
            .on_press(move || open(kind))
            .child(
                image(icon)
                    .size(px(20.0))
                    .tint_color(move || theme().primary),
            )
            .child(
                view()
                    .flex_col()
                    .child(
                        text(kind.label())
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().foreground),
                    )
                    .child(
                        text(description)
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .into_element()
    };
    let open: std::rc::Rc<dyn Fn(Filter)> = std::rc::Rc::new(open);
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Add to Money",
            "Choose what you want to add.",
            close,
        )
        .width(420.0)
        .footer_visible(false)
        .body(
            view()
                .w_full()
                .flex_col()
                .gap(px(8.0))
                .p(px(20.0))
                .child(row(
                    Filter::Subscription,
                    asset!("icons/subscriptions.svg"),
                    "A recurring service like Netflix.",
                    &open,
                ))
                .child(row(
                    Filter::Budget,
                    asset!("icons/wallet.svg"),
                    "A recurring budget you plan for.",
                    &open,
                ))
                .child(row(
                    Filter::Transaction,
                    asset!("icons/transactions.svg"),
                    "A one-off payment.",
                    &open,
                ))
                .child(row(
                    Filter::Debt,
                    asset!("icons/debts.svg"),
                    "Money you owe and pay down.",
                    &open,
                ))
                .into_element(),
        ),
    )
    .into_element()
}
