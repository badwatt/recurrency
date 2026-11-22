//! Subscriptions list: searchable cards grouped by active state.

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use uuid::Uuid;

use crate::api::{self, SubscriptionsData};
use crate::auth::AuthSession;
use crate::format::format_currency;
use crate::models::Subscription;
use crate::theme as design;
use crate::ui;

const TITLE: &str = "Subscriptions";
const SUBTITLE: &str = "Track and manage recurring payments";

pub fn subscriptions(auth: AuthSession) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let load_currency = currency.clone();
    let data = resource(
        || Some(()),
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            let currency = load_currency.clone();
            async move { api::load_subscriptions(server, token, currency).await }
        },
    );
    let search = signal(String::new());
    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(ui::page_header(TITLE, SUBTITLE))
        .child(toolbar(data, search))
        .child(
            when(loading, || loading_card()).otherwise(move || {
                let value = render_currency.clone();
                when(has_error, move || {
                    error_card(&data.error().unwrap_or_default(), data)
                })
                .otherwise(move || content(data, search, value.clone()))
            }),
        )
        .into_element()
}

fn toolbar(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
) -> impl IntoElement {
    let count = move || filtered(data, search).len();
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .child(
            input()
                .value(search)
                .on_change(move |value: &str| search.set(value.to_string()))
                .placeholder("Search")
                .auto_fill(AutoFill::Off)
                .h(px(40.0))
                .w(px(320.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            text(move || ui::count_label(count(), "subscription", "subscriptions"))
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn content(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
    currency: String,
) -> AnyElement {
    let active = move || {
        filtered(data, search)
            .into_iter()
            .filter(|subscription| subscription.is_active)
            .collect::<Vec<_>>()
    };
    let inactive = move || {
        filtered(data, search)
            .into_iter()
            .filter(|subscription| !subscription.is_active)
            .collect::<Vec<_>>()
    };
    let has_inactive = !inactive().is_empty();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(section(
            "ACTIVE",
            active,
            data,
            currency.clone(),
        ))
        .when(has_inactive, move |page| {
            page.child(section(
                "INACTIVE",
                inactive,
                data,
                currency.clone(),
            ))
        })
        .into_element()
}

fn section(
    label: &'static str,
    items: impl Fn() -> Vec<Subscription> + Clone + 'static,
    data: Resource<SubscriptionsData, String>,
    currency: String,
) -> impl IntoElement {
    let count_items = items.clone();
    let currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    text(label)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(move || count_items().len().to_string())
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .px(px(6.0))
                        .py(px(1.0))
                        .rounded(px(999.0))
                        .bg(move || theme().muted),
                ),
        )
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
                .gap(px(16.0))
                .child(each(
                    items,
                    |subscription| subscription.id,
                    move |subscription, _| card(subscription.get(), data, currency.clone()),
                )),
        )
}

fn filtered(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
) -> Vec<Subscription> {
    let data = data.get().unwrap_or_default();
    filter_subscriptions(&data.subscriptions, &search.get(), &data.accounts)
}

fn filter_subscriptions(
    subscriptions: &[Subscription],
    query: &str,
    accounts: &[crate::models::Account],
) -> Vec<Subscription> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Subscription> = subscriptions
        .iter()
        .filter(|subscription| {
            if query.is_empty() {
                return true;
            }
            let account = accounts
                .iter()
                .find(|account| account.id == subscription.account_id)
                .map_or("", |account| account.name.as_str());
            subscription.name.to_lowercase().contains(&query)
                || account.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    result.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    result
}

fn card(
    subscription: Subscription,
    data: Resource<SubscriptionsData, String>,
    currency: String,
) -> impl IntoElement {
    let color = account_color(data, subscription.account_id);
    let account = account_name(data, subscription.account_id);
    let badge_bg_color = color;
    let name = subscription.name;
    let amount = {
        let rates = data.get().unwrap_or_default().rates;
        format_currency(rates.convert(subscription.amount, &subscription.currency), &currency)
    };
    let next = subscription
        .next_billing_date
        .map_or_else(|| "No next date".to_string(), |date| format!("Next: {}", date.format("%d/%m/%Y")));
    let active = subscription.is_active;
    let badge = if active { "Active" } else { "Inactive" };
    let badge_bg = if active {
        design::success().opacity(0.15)
    } else {
        theme().muted
    };
    let badge_text = if active {
        design::success()
    } else {
        theme().muted_foreground
    };

    view()
        .w_full()
        .h(px(240.0))
        .p(px(20.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .flex_col()
        .justify_between()
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .justify_between()
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
                                .size(px(18.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    text(badge)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || badge_text)
                        .px(px(7.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(move || badge_bg),
                ),
        )
        .child(
            view()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(name)
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground)
                        .number_of_lines(1),
                )
                .child(
                    text(next)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_end()
                .justify_between()
                .gap(px(8.0))
                .child(
                    text(amount)
                        .text_size(px(24.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded(px(6.0))
                        .bg(move || theme().muted)
                        .child(
                            view()
                                .h(px(12.0))
                                .w(px(12.0))
                                .rounded(px(3.0))
                                .bg(move || rgba((parse_hex(&badge_bg_color) << 8) | 0x80)),
                        )
                        .child(
                            text(account)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
}

fn account_name(data: Resource<SubscriptionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .map_or_else(|| "No account".to_string(), |account| account.name)
}

fn account_color(data: Resource<SubscriptionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .and_then(|account| account.color)
        .unwrap_or_else(|| "#6b7280".to_string())
}


fn parse_hex(value: &str) -> u32 {
    u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0x6b7280)
}

fn loading_card() -> AnyElement {
    ui::surface()
        .p(px(28.0))
        .child(
            text("Loading subscriptions…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<SubscriptionsData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("Subscriptions could not be loaded.")
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
    use crate::models::{AnnualDate, Recurrence};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn subscription(name: &str, account_id: Uuid, active: bool) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: name.to_string(),
            amount: Decimal::from(10),
            currency: "EUR".to_string(),
            recurrence: Recurrence::default(),
            start_date: date(2026, 1, 1),
            next_billing_date: Some(date(2026, 11, 23)),
            end_date: None,
            is_active: active,
            logo_url: None,
        }
    }

    #[test]
    fn search_matches_names_and_accounts_case_insensitively() {
        let account_id = Uuid::new_v4();
        let accounts = vec![crate::models::Account {
            id: account_id,
            name: "Everyday".to_string(),
            icon: None,
            color: None,
            is_default: true,
        }];
        let items = vec![
            subscription("Spotify", account_id, true),
            subscription("Netflix", Uuid::new_v4(), true),
        ];
        assert_eq!(
            filter_subscriptions(&items, "spot", &accounts)[0].name,
            "Spotify"
        );
        assert_eq!(
            filter_subscriptions(&items, "every", &accounts).len(),
            1
        );
        assert_eq!(filter_subscriptions(&items, "", &accounts).len(), 2);
    }

    #[test]
    fn inactive_subscriptions_sort_after_active_ones() {
        let account_id = Uuid::new_v4();
        let items = vec![
            subscription("Aardvark", account_id, false),
            subscription("Zebra", account_id, true),
        ];
        let filtered = filter_subscriptions(&items, "", &[]);
        assert_eq!(filtered[0].name, "Zebra");
        assert!(!filtered[1].is_active);
    }

    #[test]
    fn subscription_serde_keeps_next_date() {
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "account_id":"00000000-0000-0000-0000-000000000002",
            "category_id":"00000000-0000-0000-0000-000000000003",
            "name":"Spotify",
            "amount":"10.99",
            "currency":"EUR",
            "recurrence":{"interval":1,"unit":"month","days":[23],"weekdays":[],"dates":[]},
            "start_date":"2026-01-01",
            "next_billing_date":"2026-11-23",
            "end_date":null,
            "is_active":true,
            "logo_url":null
        }"#;
        let subscription: Subscription = serde_json::from_str(json).expect("subscription parses");
        assert_eq!(subscription.next_billing_date, Some(date(2026, 11, 23)));
        assert_eq!(
            subscription.recurrence,
            Recurrence {
                interval: 1,
                unit: "month".to_string(),
                days: vec![23],
                weekdays: Vec::new(),
                dates: Vec::<AnnualDate>::new(),
            }
        );
        assert_eq!(subscription.amount, Decimal::new(1099, 2));
    }
}
