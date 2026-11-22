//! Monthly reports: summary, category split and recent trends.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::theme;
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::api::{self, ReportsData};
use crate::auth::AuthSession;
use crate::format::format_currency;
use crate::models::{Category, ReportResponse};
use crate::theme as design;
use crate::ui;

const TITLE: &str = "Reports";
const SUBTITLE: &str = "Analyze your spending patterns";

pub fn reports(auth: AuthSession) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let today = Local::now().date_naive();
    let year = signal(today.year());
    let month = signal(today.month());
    let resource_server = server.clone();
    let resource_token = token.clone();
    let data = resource(
        move || Some((year.get(), month.get())),
        move |(load_year, load_month), _| {
            let server = resource_server.clone();
            let token = resource_token.clone();
            async move { api::load_report(server, token, load_year, load_month).await }
        },
    );
    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header())
        .child(navigation(year, month, today))
        .child(
            when(loading, || loading_card()).otherwise(move || {
                let currency = render_currency.clone();
                when(has_error, move || {
                    error_card(&data.error().unwrap_or_default(), data)
                })
                .otherwise(move || content(data, currency.clone()))
            }),
        )
        .into_element()
}

fn header() -> impl IntoElement {
    view().w_full().child(ui::page_header(TITLE, SUBTITLE))
}

fn navigation(year: Signal<i32>, month: Signal<u32>, today: NaiveDate) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .child(ui::action_button("Previous", false, move || {
            let (next_year, next_month) = shift_month(year.get(), month.get(), -1);
            year.set(next_year);
            month.set(next_month);
        }))
        .child(
            text(move || month_label(year.get(), month.get()))
                .text_size(px(20.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(ui::action_button("Next", false, move || {
            let (next_year, next_month) = shift_month(year.get(), month.get(), 1);
            year.set(next_year);
            month.set(next_month);
        }))
        .child(ui::action_button("Today", false, move || {
            year.set(today.year());
            month.set(today.month());
        }))
}

fn content(data: Resource<ReportsData, String>, currency: String) -> AnyElement {
    let report = move || data.get().unwrap_or_default().report;
    let total_currency = currency.clone();
    let subscriptions_currency = currency.clone();
    let transactions_currency = currency.clone();
    let average_currency = currency.clone();
    let total = move || {
        format_currency(
            report().monthly_summary.overall_total,
            &total_currency,
        )
    };
    let subscriptions = move || {
        format_currency(
            report().monthly_summary.subscriptions_total,
            &subscriptions_currency,
        )
    };
    let transactions = move || {
        format_currency(
            report().monthly_summary.transactions_total,
            &transactions_currency,
        )
    };
    let average = move || {
        let value = average_trend(&report());
        format_currency(value, &average_currency)
    };
    let category_currency = currency.clone();
    let trend_currency = currency.clone();

    view()
        .w_full()
        .flex_col()
        .gap(px(20.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .justify_end()
                .child(ui::action_button("Export CSV", false, {
                    let currency = currency.clone();
                    move || {
                        let Some(snapshot) = data.get() else {
                            return;
                        };
                        let csv = build_csv(&snapshot.report, &currency);
                        spawn(async move {
                            if let Ok(Some(path)) = pick_save_path(
                                SaveDialog::new()
                                    .file_name("recurrency-report.csv")
                                    .filter(FileFilter::new("CSV", ["csv"])),
                            )
                            .await
                            {
                                let _ = std::fs::write(path, csv);
                            }
                        });
                    }
                })),
        )
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
                .gap(px(12.0))
                .child(summary_card("Total spending", total, "This month"))
                .child(summary_card("Subscriptions", subscriptions, "Recurring"))
                .child(summary_card("One-time", transactions, "Transactions"))
                .child(summary_card("Avg / month", average, "Recent trends")),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .gap(px(16.0))
                .child(category_card(data, &category_currency))
                .child(trend_card(data, &trend_currency)),
        )
        .into_element()
}

fn summary_card(
    label: &'static str,
    value: impl Fn() -> String + 'static,
    caption: &'static str,
) -> impl IntoElement {
    view()
        .w_full()
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(18.0))
        .gap(px(4.0))
        .child(
            text(label)
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            text(value)
                .text_size(px(24.0))
                .font_family(design::MONO)
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(caption)
                .text_size(px(10.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn category_card(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    let categories = move || data.get().unwrap_or_default().report.category_breakdown;
    view()
        .flex_grow(1.2)
        .min_w(px(0.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(12.0))
        .child(
            text("By category")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .flex_col()
                .gap(px(10.0))
                .child(each(
                    categories,
                    |category| category.category_name.clone(),
                    move |category, _| {
                        let category = category.get();
                        let color = category_color(data, &category.category_name);
                        let amount = format_currency(category.total, &currency);
                        let percent = category.percentage.clamp(0.0, 100.0) as f32;
                        let color_text = color.clone();
                        let color_bg = color;
                        view()
                            .w_full()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        view()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                view()
                                                    .h(px(8.0))
                                                    .w(px(8.0))
                                                    .rounded(px(999.0))
                                                    .bg(move || rgb(parse_hex(&color_bg))),
                                            )
                                            .child(
                                                text(category.category_name)
                                                    .text_size(px(12.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().foreground),
                                            ),
                                    )
                                    .child(
                                        text(amount)
                                            .text_size(px(12.0))
                                            .font_family(design::MONO)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(move || theme().foreground),
                                    ),
                            )
                            .child(
                                view()
                                    .h(px(5.0))
                                    .w_full()
                                    .rounded(px(999.0))
                                    .bg(move || theme().muted)
                                    .overflow_hidden()
                                    .flex_row()
                                    .child(
                                        view()
                                            .h_full()
                                            .flex_grow(percent.max(0.01))
                                            .bg(move || rgb(parse_hex(&color_text))),
                                    )
                                    .child(view().h_full().flex_grow((100.0 - percent).max(0.01))),
                            )
                            .child(
                                text(format!("{} items · {:.1}%", category.count, category.percentage))
                                    .text_size(px(10.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                    },
                )),
        )
}

fn trend_card(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    let trends = move || {
        let mut trends = data.get().unwrap_or_default().report.recent_trends;
        trends.reverse();
        trends
    };
    view()
        .flex_grow(0.8)
        .min_w(px(0.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(12.0))
        .child(
            text("Spending trends")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .flex_col()
                .gap(px(8.0))
                .child(each(
                    trends,
                    |trend| trend.period.clone(),
                    move |trend, _| {
                        let trend = trend.get();
                        let max = data
                            .get()
                            .unwrap_or_default()
                            .report
                            .recent_trends
                            .iter()
                            .map(|item| item.total)
                            .max()
                            .unwrap_or(Decimal::ONE);
                        let ratio = if max > Decimal::ZERO {
                            (trend.total / max * Decimal::from(100))
                                .to_string()
                                .parse::<f32>()
                                .unwrap_or(0.0)
                                .clamp(0.0, 100.0)
                        } else {
                            0.0
                        };
                        let amount = format_currency(trend.total, &currency);
                        view()
                            .w_full()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                view()
                                    .flex_row()
                                    .justify_between()
                                    .child(
                                        text(trend.period)
                                            .text_size(px(11.0))
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text(amount)
                                            .text_size(px(11.0))
                                            .font_family(design::MONO)
                                            .text_color(move || theme().foreground),
                                    ),
                            )
                            .child(
                                view()
                                    .h(px(5.0))
                                    .w_full()
                                    .rounded(px(999.0))
                                    .bg(move || theme().muted)
                                    .overflow_hidden()
                                    .flex_row()
                                    .child(
                                        view()
                                            .h_full()
                                            .flex_grow(ratio.max(0.01))
                                            .bg(move || theme().primary),
                                    )
                                    .child(view().h_full().flex_grow((100.0 - ratio).max(0.01))),
                            )
                    },
                )),
        )
}

fn build_csv(report: &ReportResponse, currency: &str) -> String {
    let mut csv = String::from("Period,Total\n");
    for trend in &report.recent_trends {
        csv.push_str(&format!("{},{}\n", trend.period, trend.total));
    }
    csv.push_str("\nCategory,Amount,Count,Percentage\n");
    for category in &report.category_breakdown {
        csv.push_str(&format!(
            "{},{},{},{:.1}%\n",
            category.category_name, category.total, category.count, category.percentage
        ));
    }
    csv.push_str(&format!("\nCurrency,{currency}\n"));
    csv
}

fn average_trend(report: &ReportResponse) -> Decimal {
    if report.recent_trends.is_empty() {
        return report.monthly_summary.overall_total;
    }
    let sum: Decimal = report.recent_trends.iter().map(|trend| trend.total).sum();
    sum / Decimal::from(report.recent_trends.len() as i64)
}

fn category_color(data: Resource<ReportsData, String>, name: &str) -> String {
    data.get()
        .unwrap_or_default()
        .categories
        .into_iter()
        .find(|category: &Category| category.name == name)
        .and_then(|category| category.color)
        .unwrap_or_else(|| "#6b7280".to_string())
}

fn parse_hex(value: &str) -> u32 {
    u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0x6b7280)
}

fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let total = year * 12 + month as i32 - 1 + delta;
    (total.div_euclid(12), (total.rem_euclid(12) + 1) as u32)
}

fn month_label(year: i32, month: u32) -> String {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map_or_else(String::new, |date| date.format("%B %Y").to_string())
}

fn loading_card() -> AnyElement {
    ui::surface()
        .p(px(28.0))
        .child(
            text("Loading reports…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<ReportsData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("Reports could not be loaded.")
                .text_size(px(15.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(message)
                .text_size(px(13.0))
                .text_color(move || theme().destructive),
        )
        .child(ui::action_button("Try again", false, move || data.refetch()))
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MonthlySummary, SpendingTrend};

    #[test]
    fn average_uses_recent_trends() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary::default(),
            category_breakdown: Vec::new(),
            recent_trends: vec![
                SpendingTrend {
                    period: "11/26".to_string(),
                    total: Decimal::from(30),
                },
                SpendingTrend {
                    period: "10/26".to_string(),
                    total: Decimal::from(10),
                },
            ],
        };
        assert_eq!(average_trend(&report), Decimal::from(20));
    }

    #[test]
    fn average_falls_back_to_the_summary() {
        let mut report = ReportResponse::default();
        report.monthly_summary.overall_total = Decimal::from(42);
        assert_eq!(average_trend(&report), Decimal::from(42));
    }

    #[test]
    fn csv_contains_trends_and_categories() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary::default(),
            category_breakdown: vec![crate::models::CategoryBreakdown {
                category_id: None,
                category_name: "Streaming".to_string(),
                total: Decimal::from(10),
                count: 1,
                percentage: 100.0,
            }],
            recent_trends: vec![SpendingTrend {
                period: "11/26".to_string(),
                total: Decimal::from(10),
            }],
        };
        let csv = build_csv(&report, "EUR");
        assert!(csv.contains("Streaming"));
        assert!(csv.contains("11/26"));
        assert!(csv.contains("Currency,EUR"));
    }

    #[test]
    fn month_navigation_crosses_years() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(month_label(2026, 11), "November 2026");
    }
}
