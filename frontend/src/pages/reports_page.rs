use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::api::reports;
use crate::api::categories;
use crate::models::report::*;
use crate::utils::format::format_currency;
use chrono::{Datelike, Local};

const MONTH_OPTIONS: &[(&str, &str)] = &[
    ("1", "January"), ("2", "February"), ("3", "March"),
    ("4", "April"), ("5", "May"), ("6", "June"),
    ("7", "July"), ("8", "August"), ("9", "September"),
    ("10", "October"), ("11", "November"), ("12", "December"),
];

fn report_years() -> Vec<i32> {
    let this_year = Local::now().year();
    vec![this_year - 2, this_year - 1, this_year]
}

/// Build CSV from report data.
pub fn build_csv(report: &ReportResponse) -> String {
    let mut csv = String::from("Category,Amount,Count,Percentage\n");
    for cat in &report.category_breakdown {
        csv.push_str(&format!(
            "{},{},{},{:.1}%\n",
            cat.category_name, cat.total, cat.count, cat.percentage
        ));
    }
    csv
}

/// Get category style (color, icon) by name from category list.
pub fn category_style_by_name<'a>(
    categories: &'a [crate::models::category::Category],
    name: &str,
) -> (&'a str, &'a str) {
    categories.iter()
        .find(|c| c.name == name)
        .map(|c| {
            (c.color.as_deref().unwrap_or("#6b7280"), c.icon.as_deref().unwrap_or("Tag"))
        })
        .unwrap_or(("#6b7280", "Tag"))
}

#[component]
pub fn ReportsPage() -> Element {
    let now = Local::now();
    let mut selected_year = use_signal(|| now.year().to_string());
    let mut selected_month = use_signal(|| now.month().to_string());
    let mut report = use_signal(|| None::<ReportResponse>);
    let mut is_loading = use_signal(|| true);

    // Load categories
    use_future(move || async move {
        if !CATEGORIES().categories.is_empty() { return; }
        CATEGORIES.write().set_loading();
        match categories::list().await {
            Ok(data) => CATEGORIES.write().set_data(data),
            Err(e) => CATEGORIES.write().set_error(e.to_string()),
        }
    });

    // Load report
    let year_val = selected_year();
    let month_val = selected_month();
    use_future(move || {
        let y = year_val.clone();
        let m = month_val.clone();
        async move {
        is_loading.set(true);
        let y: i32 = y.parse().unwrap_or(now.year());
        let m: u32 = m.parse().unwrap_or(now.month());
        match reports::get_summary(y, m).await {
            Ok(data) => report.set(Some(data)),
            Err(e) => log::error!("Failed to fetch report: {}", e),
        }
        is_loading.set(false);
        }
    });

    let currency = AUTH().currency().to_string();
    let cats = CATEGORIES();
    let years = report_years();

    let handle_export = move |_| {
        if let Some(r) = report() {
            let csv = build_csv(&r);
            let _ = dioxus::document::eval(&format!(
                "const blob = new Blob([`{}`], {{type: 'text/csv'}}); const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = 'recurrency-report-{}-{}.csv'; a.click(); URL.revokeObjectURL(url);",
                csv.replace('`', "\\`").replace('\n', "\\n"),
                selected_year(),
                selected_month(),
            ));
        }
    };

    rsx! {
        div { class: "p-4 md:p-6 space-y-6 overflow-x-hidden",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Reports" }
                    p { class: "text-muted-foreground", "Analyze your spending patterns" }
                }
                div { class: "flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-3",
                    div { class: "flex items-center gap-2",
                        select {
                            value: "{selected_month}",
                            onchange: move |e| selected_month.set(e.value()),
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                            for (val, label) in MONTH_OPTIONS {
                                option { value: "{val}", "{label}" }
                            }
                        }
                        select {
                            value: "{selected_year}",
                            onchange: move |e| selected_year.set(e.value()),
                            class: "flex h-9 w-20 rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                            for y in &years {
                                option { value: "{y}", "{y}" }
                            }
                        }
                    }
                    button {
                        onclick: handle_export,
                        class: "flex items-center justify-center gap-1.5 px-3 py-2 text-sm border border-border rounded-md hover:bg-accent",
                        "⬇ Export CSV"
                    }
                }
            }

            {if is_loading() {
                rsx! {
                    div { class: "flex h-64 items-center justify-center",
                        div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
                    }
                }
            } else if let Some(r) = report() {
                rsx! {
                    div { class: "space-y-6",
                        // Summary Cards
                        div { class: "grid gap-4 md:grid-cols-4",
                            // Total Spending
                            div { class: "rounded-lg border border-border bg-card p-6",
                                div { class: "flex items-center justify-between pb-2",
                                    span { class: "text-sm font-medium text-muted-foreground", "Total Spending" }
                                    span { class: "text-muted-foreground", "💰" }
                                }
                                p { class: "text-2xl font-bold",
                                    {format_currency(r.monthly_summary.overall_total, &currency)}
                                }
                                p { class: "text-xs text-muted-foreground",
                                    {MONTH_OPTIONS.iter().find(|(v, _)| *v == selected_month()).map(|(_, l)| *l).unwrap_or("")}
                                    " {selected_year()}"
                                }
                            }
                            // Subscriptions
                            div { class: "rounded-lg border border-border bg-card p-6",
                                div { class: "flex items-center justify-between pb-2",
                                    span { class: "text-sm font-medium text-muted-foreground", "Subscriptions" }
                                    span { class: "text-muted-foreground", "💳" }
                                }
                                p { class: "text-2xl font-bold",
                                    {format_currency(r.monthly_summary.subscriptions_total, &currency)}
                                }
                                p { class: "text-xs text-muted-foreground",
                                    "{r.monthly_summary.subscriptions_count} active"
                                }
                            }
                            // One-time
                            div { class: "rounded-lg border border-border bg-card p-6",
                                div { class: "flex items-center justify-between pb-2",
                                    span { class: "text-sm font-medium text-muted-foreground", "One-time" }
                                    span { class: "text-muted-foreground", "🧾" }
                                }
                                p { class: "text-2xl font-bold",
                                    {format_currency(r.monthly_summary.transactions_total, &currency)}
                                }
                                p { class: "text-xs text-muted-foreground",
                                    "{r.monthly_summary.transactions_count} transactions"
                                }
                            }
                            // Categories
                            div { class: "rounded-lg border border-border bg-card p-6",
                                div { class: "flex items-center justify-between pb-2",
                                    span { class: "text-sm font-medium text-muted-foreground", "Categories" }
                                    span { class: "text-muted-foreground", "📈" }
                                }
                                p { class: "text-2xl font-bold",
                                    "{r.category_breakdown.len()}"
                                }
                                p { class: "text-xs text-muted-foreground", "with expenses" }
                            }
                        }

                        // Spending Trends (simplified text table, no chart lib)
                        div { class: "grid gap-6 md:grid-cols-2",
                            div { class: "rounded-lg border border-border bg-card p-6",
                                h3 { class: "text-lg font-semibold mb-4", "Spending Trends" }
                                if r.recent_trends.is_empty() {
                                    p { class: "text-muted-foreground text-center py-8", "No trend data available" }
                                } else {
                                    div { class: "space-y-2",
                                        for trend in &r.recent_trends {
                                            {let trend_total_str = format_currency(trend.total, &currency);
                                             let pct = {
                                                 let max_total: rust_decimal::Decimal = r.recent_trends.iter().map(|t| t.total).max().unwrap_or(rust_decimal::Decimal::ONE);
                                                 if max_total > rust_decimal::Decimal::ZERO {
                                                     let val = trend.total / max_total * rust_decimal::Decimal::from(100);
                                                     val.to_string()
                                                 } else { "0".to_string() }
                                             };
                                             rsx! {
                                                div { key: "{trend.period}", class: "flex items-center justify-between py-1",
                                                    span { class: "text-sm", "{trend.period}" }
                                                    span { class: "text-sm font-medium", "{trend_total_str}" }
                                                }
                                                div { class: "h-2 bg-muted rounded-full overflow-hidden",
                                                    div {
                                                        class: "h-full rounded-full bg-primary",
                                                        style: "width: {pct}%",
                                                    }
                                                }
                                            }}
                                        }
                                    }
                                }
                            }

                            // Category Breakdown (pie-like display)
                            div { class: "rounded-lg border border-border bg-card p-6",
                                h3 { class: "text-lg font-semibold mb-4", "By Category" }
                                if r.category_breakdown.is_empty() {
                                    p { class: "text-muted-foreground text-center py-8", "No category data available" }
                                } else {
                                    div { class: "space-y-2",
                                        for cat in r.category_breakdown.iter().take(5) {
                                            {let (color, _icon) = category_style_by_name(&cats.categories, &cat.category_name);
                                             rsx! {
                                                div { key: "{cat.category_name}", class: "flex items-center gap-2",
                                                    div {
                                                        class: "w-6 h-6 rounded-full flex items-center justify-center",
                                                        style: "background-color: {color}20",
                                                        div { class: "w-3 h-3 rounded-full", style: "background-color: {color}" }
                                                    }
                                                    span { class: "text-sm flex-1 truncate", "{cat.category_name}" }
                                                    span { class: "text-sm font-medium", "{cat.percentage:.0}%" }
                                                }
                                            }}
                                        }
                                    }
                                }
                            }
                        }

                        // Category Details Table
                        div { class: "rounded-lg border border-border bg-card p-6",
                            h3 { class: "text-lg font-semibold mb-4", "Category Breakdown" }
                            if r.category_breakdown.is_empty() {
                                p { class: "text-muted-foreground text-center py-8", "No category data available" }
                            } else {
                                div { class: "space-y-4",
                                    for cat in &r.category_breakdown {
                                        {let (color, _icon) = category_style_by_name(&cats.categories, &cat.category_name);
                                         rsx! {
                                            div { key: "{cat.category_name}", class: "space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    div { class: "flex items-center gap-3",
                                                        div {
                                                            class: "w-8 h-8 rounded-full flex items-center justify-center",
                                                            style: "background-color: {color}20",
                                                            div { class: "w-4 h-4 rounded-full", style: "background-color: {color}" }
                                                        }
                                                        span { class: "font-medium", "{cat.category_name}" }
                                                    }
                                                    div { class: "flex items-center gap-4",
                                                        span { class: "text-muted-foreground", "{cat.count} items" }
                                                        span { class: "font-bold",
                                                            {format_currency(cat.total, &currency)}
                                                        }
                                                    }
                                                }
                                                div { class: "h-2 bg-muted rounded-full overflow-hidden",
                                                    div {
                                                        class: "h-full rounded-full",
                                                        style: "width: {cat.percentage}%; background-color: {color}",
                                                    }
                                                }
                                            }
                                        }}
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "text-center text-muted-foreground p-6",
                        "No data available for this period"
                    }
                }
            }}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::models::category::Category;
    use uuid::Uuid;
    use chrono::{DateTime, Utc};

    fn make_cat(name: &str, color: &str, icon: &str) -> Category {
        Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: name.into(),
            icon: Some(icon.into()),
            color: Some(color.into()),
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn build_csv_format() {
        let report = ReportResponse {
            monthly_summary: crate::models::report::MonthlySummary {
                year: 2024, month: 1,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![
                crate::models::report::CategoryBreakdown {
                    category_id: None,
                    category_name: "Food".into(),
                    total: dec!(100),
                    count: 5,
                    percentage: 66.7,
                },
            ],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        assert!(csv.contains("Category,Amount,Count,Percentage"));
        assert!(csv.contains("Food"));
        assert!(csv.contains("66.7%"));
    }

    #[test]
    fn category_style_by_name_found() {
        let cats = vec![make_cat("Food", "#ef4444", "Coffee")];
        let (color, icon) = category_style_by_name(&cats, "Food");
        assert_eq!(color, "#ef4444");
        assert_eq!(icon, "Coffee");
    }

    #[test]
    fn category_style_by_name_not_found() {
        let cats: Vec<Category> = vec![];
        let (color, icon) = category_style_by_name(&cats, "Missing");
        assert_eq!(color, "#6b7280");
        assert_eq!(icon, "Tag");
    }

    #[test]
    fn category_style_fallback_when_null() {
        let mut cats = vec![make_cat("Test", "#abc", "Star")];
        cats[0].color = None;
        cats[0].icon = None;
        let (color, icon) = category_style_by_name(&cats, "Test");
        assert_eq!(color, "#6b7280");
        assert_eq!(icon, "Tag");
    }

    #[test]
    fn month_options_count() {
        assert_eq!(MONTH_OPTIONS.len(), 12);
    }
}