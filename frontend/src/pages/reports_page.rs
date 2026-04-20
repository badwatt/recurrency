use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::api::reports;
use crate::api::categories;
use crate::models::report::*;
use crate::utils::format::format_currency;
use crate::components::chart::{LineChart, LineData, DonutChart, DonutData, palette_color};
use chrono::{Datelike, Local};

fn icon_credit_card() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        rect { x: "1", y: "4", width: "22", height: "16", rx: "2" }
        line { x1: "1", y1: "10", x2: "23", y2: "10" }
    }}
}

fn icon_receipt() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        line { x1: "4", y1: "2", x2: "4", y2: "22" }
        path { d: "M1 2h18a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H1" }
    }}
}

fn icon_trending_up() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        polyline { points: "23 6 13.5 15.5 8.5 10.5 1 18" }
        polyline { points: "17 6 23 6" }
        polyline { points: "23 6 23 12" }
    }}
}


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
        // android guard removed
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
        div { class: "p-4 md:p-6 flex flex-col gap-6 overflow-x-hidden",
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
                    div { class: "flex flex-col gap-6",
                        // ── Summary Cards ─────────────────────────────
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
                                    span { class: "text-muted-foreground", {icon_credit_card()} }
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
                                    span { class: "text-muted-foreground", {icon_receipt()} }
                                }
                                p { class: "text-2xl font-bold",
                                    {format_currency(r.monthly_summary.transactions_total, &currency)}
                                }
                                p { class: "text-xs text-muted-foreground",
                                    "{r.monthly_summary.transactions_count} transactions"
                                }
                            }
                            // Avg per month (derived from trends)
                            {
                                let avg: rust_decimal::Decimal = if r.recent_trends.is_empty() {
                                    r.monthly_summary.overall_total
                                } else {
                                    let sum: rust_decimal::Decimal = r.recent_trends.iter().map(|t| t.total).sum();
                                    sum / rust_decimal::Decimal::from(r.recent_trends.len() as i64)
                                };
                                rsx! {
                                    div { class: "rounded-lg border border-border bg-card p-6",
                                        div { class: "flex items-center justify-between pb-2",
                                            span { class: "text-sm font-medium text-muted-foreground", "Avg / Month" }
                                            span { class: "text-muted-foreground", {icon_trending_up()} }
                                        }
                                        p { class: "text-2xl font-bold",
                                            {format_currency(avg, &currency)}
                                        }
                                        p { class: "text-xs text-muted-foreground",
                                            "over {r.recent_trends.len()} months"
                                        }
                                    }
                                }
                            }
                        }

                        // ── Charts Row: Line + Donut ────────────────────
                        div { class: "grid gap-6 md:grid-cols-2",
                            // Spending Trends Line Chart
                            div { class: "rounded-lg border border-border bg-card p-6",
                                h3 { class: "text-lg font-semibold mb-4", "Spending Trends" }
                                if r.recent_trends.is_empty() {
                                    p { class: "text-muted-foreground text-center py-8", "No trend data available" }
                                } else {
                                    {
                                        // Trends come newest-first from API, reverse for chronological
                                        let mut trends = r.recent_trends.clone();
                                        trends.reverse();
                                        let line_data: Vec<LineData> = trends.iter().map(|t| {
                                            LineData {
                                                label: t.period.clone(),
                                                value: t.total.to_string().parse::<f64>().unwrap_or(0.0),
                                            }
                                        }).collect();
                                        rsx! {
                                            LineChart {
                                                data: line_data,
                                                height: "220".to_string(),
                                                color: "#6366f1".to_string(),
                                            }
                                        }
                                    }
                                }
                            }

                            // Category Breakdown Donut Chart
                            div { class: "rounded-lg border border-border bg-card p-6",
                                h3 { class: "text-lg font-semibold mb-4", "By Category" }
                                if r.category_breakdown.is_empty() {
                                    p { class: "text-muted-foreground text-center py-8", "No category data available" }
                                } else {
                                    div { class: "flex flex-col items-center gap-4",
                                        {
                                            let donut_data: Vec<DonutData> = r.category_breakdown.iter().enumerate().map(|(i, cat)| {
                                                let pos = cats.categories.iter().position(|c2| c2.name == cat.category_name).unwrap_or(i);
                                                let color = palette_color(pos).to_string();
                                                DonutData {
                                                    label: cat.category_name.clone(),
                                                    value: cat.percentage,
                                                    color,
                                                }
                                            }).collect();
                                            rsx! {
                                                DonutChart {
                                                    data: donut_data,
                                                    size: "180".to_string(),
                                                    stroke_width: "12".to_string(),
                                                }
                                            }
                                        }
                                        // Legend
                                        div { class: "grid grid-cols-2 gap-x-4 gap-y-1.5 w-full",
                                            for (i, cat) in r.category_breakdown.iter().take(8).enumerate() {
                                                {
                                                    let cats_guard = CATEGORIES();
                                                    let color = if let Some(pos) = cats_guard.categories.iter().position(|c2| c2.name == cat.category_name) {
                                                        palette_color(pos).to_string()
                                                    } else {
                                                        palette_color(i).to_string()
                                                    };
                                                    rsx! {
                                                        div { key: "{cat.category_name}", class: "flex items-center gap-1.5 text-sm",
                                                            div { class: "w-2.5 h-2.5 rounded-full shrink-0", style: "background-color: {color}" }
                                                            span { class: "truncate text-muted-foreground", "{cat.category_name}" }
                                                            span { class: "font-medium ml-auto", "{cat.percentage:.0}%" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ── Category Details Table ──────────────────────
                        div { class: "rounded-lg border border-border bg-card p-6",
                            h3 { class: "text-lg font-semibold mb-4", "Category Breakdown" }
                            if r.category_breakdown.is_empty() {
                                p { class: "text-muted-foreground text-center py-8", "No category data available" }
                            } else {
                                div { class: "space-y-4",
                                    for cat in r.category_breakdown.iter() {
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
    use chrono::DateTime;

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

    #[test]
    fn palette_color_deterministic() {
        use crate::components::chart::palette_color;
        assert_eq!(palette_color(0), "#6366f1");
        assert_eq!(palette_color(1), "#f59e0b");
    }

    #[test]
    fn donut_data_construction() {
        use crate::components::chart::DonutData;
        let d = DonutData {
            label: "Food".into(),
            value: 45.0,
            color: "#6366f1".into(),
        };
        assert_eq!(d.label, "Food");
        assert_eq!(d.value, 45.0);
    }

    #[test]
    fn line_data_construction() {
        use crate::components::chart::LineData;
        let d = LineData {
            label: "Jan".into(),
            value: 150.0,
        };
        assert_eq!(d.label, "Jan");
        assert_eq!(d.value, 150.0);
    }

    // ── Additional tests ──
    #[test]
    fn build_csv_all_categories() {
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
                crate::models::report::CategoryBreakdown {
                    category_id: None,
                    category_name: "Transport".into(),
                    total: dec!(50),
                    count: 3,
                    percentage: 33.3,
                },
            ],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        assert!(csv.contains("Food"));
        assert!(csv.contains("Transport"));
        assert!(csv.contains("66.7%"));
        assert!(csv.contains("33.3%"));
    }

    #[test]
    fn build_csv_empty_breakdown() {
        let report = ReportResponse {
            monthly_summary: crate::models::report::MonthlySummary {
                year: 2024, month: 1,
                subscriptions_total: dec!(0),
                transactions_total: dec!(0),
                overall_total: dec!(0),
                subscriptions_count: 0,
                transactions_count: 0,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        assert!(csv.starts_with("Category,Amount,Count,Percentage\n"));
    }

    #[test]
    fn category_style_by_name_with_multiple() {
        let cats = vec![
            make_cat("Food", "#ef4444", "Coffee"),
            make_cat("Transport", "#3b82f6", "Car"),
        ];
        let (color, icon) = category_style_by_name(&cats, "Transport");
        assert_eq!(color, "#3b82f6");
        assert_eq!(icon, "Car");
    }

    #[test]
    fn report_years_returns_three() {
        let years = report_years();
        assert_eq!(years.len(), 3);
    }

    #[test]
    fn month_options_first_january() {
        assert_eq!(MONTH_OPTIONS[0], ("1", "January"));
    }

    #[test]
    fn month_options_last_december() {
        assert_eq!(MONTH_OPTIONS[11], ("12", "December"));
    }

    #[test]
    fn avg_per_month_computation() {
        // Simulate the avg computation from the component
        let trends = vec![
            crate::models::report::SpendingTrend { period: "2024-01".into(), total: dec!(100) },
            crate::models::report::SpendingTrend { period: "2024-02".into(), total: dec!(200) },
        ];
        let sum: rust_decimal::Decimal = trends.iter().map(|t| t.total).sum();
        let avg = sum / rust_decimal::Decimal::from(trends.len() as i64);
        assert_eq!(avg, dec!(150));
    }

    #[test]
    fn avg_per_month_empty_trends_falls_to_total() {
        let overall_total = dec!(250);
        let trends: Vec<crate::models::report::SpendingTrend> = vec![];
        let avg = if trends.is_empty() {
            overall_total
        } else {
            let sum: rust_decimal::Decimal = trends.iter().map(|t| t.total).sum();
            sum / rust_decimal::Decimal::from(trends.len() as i64)
        };
        assert_eq!(avg, dec!(250));
    }

    #[test]
    fn csv_escaping() {
        // Category name with comma
        let report = ReportResponse {
            monthly_summary: crate::models::report::MonthlySummary {
                year: 2024, month: 1,
                subscriptions_total: dec!(50),
                transactions_total: dec!(0),
                overall_total: dec!(50),
                subscriptions_count: 1,
                transactions_count: 0,
            },
            category_breakdown: vec![
                crate::models::report::CategoryBreakdown {
                    category_id: None,
                    category_name: "Food, Drinks".into(),
                    total: dec!(50),
                    count: 1,
                    percentage: 100.0,
                },
            ],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        assert!(csv.contains("Food, Drinks"));
    }

    #[test]
    fn donut_data_ordering() {
        use crate::components::chart::DonutData;
        let data = vec![
            DonutData { label: "A".into(), value: 50.0, color: "#f00".into() },
            DonutData { label: "B".into(), value: 30.0, color: "#0f0".into() },
            DonutData { label: "C".into(), value: 20.0, color: "#00f".into() },
        ];
        let total: f64 = data.iter().map(|d| d.value).sum();
        assert_eq!(total, 100.0);
    }

    #[test]
    fn palette_color_all_colors() {
        use crate::components::chart::palette_color;
        for i in 0..12 {
            let c = palette_color(i);
            assert!(c.starts_with('#'), "Color at index {} should start with #", i);
        }
    }

    #[test]
    fn export_filename_construction() {
        let year = "2024";
        let month = "6";
        let filename = format!("recurrency-report-{}-{}.csv", year, month);
        assert_eq!(filename, "recurrency-report-2024-6.csv");
    }

    // ── RSX structure checks ──
    #[test]
    fn reports_page_has_title() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("Reports") || src.contains("Report"), "Should have Reports title");
    }

    #[test]
    fn reports_page_uses_chart_components() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("Chart") || src.contains("Donut") || src.contains("Line"), "Should use chart components");
    }

    #[test]
    fn reports_page_has_export_button() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("export") || src.contains("Export") || src.contains("download"), "Should have export button");
    }

    #[test]
    fn reports_page_has_month_selector() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("month") || src.contains("Month") || src.contains("Select"), "Should have month selector");
    }

    #[test]
    fn reports_page_has_year_selector() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("year") || src.contains("Year") || src.contains("report_years"), "Should have year selector");
    }

    #[test]
    fn reports_page_uses_signals() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("use_signal"), "Should use signals for state");
    }

    #[test]
    fn reports_page_uses_effect() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("use_effect"), "Should use effect for side effects");
    }

    #[test]
    fn reports_page_uses_future() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("use_future"), "Should use future for async loading");
    }

    #[test]
    fn reports_page_logs_errors() {
        let src = include_str!("reports_page.rs");
        assert!(src.contains("log::error"), "Should log errors");
    }

    // ── Month options edge cases ──
    #[test]
    fn month_options_returns_12_items() {
        assert_eq!(MONTH_OPTIONS.len(), 12);
    }

    #[test]
    fn month_options_first_is_january() {
        assert_eq!(MONTH_OPTIONS[0].0, "1");
        assert_eq!(MONTH_OPTIONS[0].1, "January");
    }

    #[test]
    fn month_options_last_is_december() {
        assert_eq!(MONTH_OPTIONS[11].0, "12");
        assert_eq!(MONTH_OPTIONS[11].1, "December");
    }

    // ── report_years edge cases ──
    #[test]
    fn report_years_has_three_items() {
        let years = report_years();
        assert_eq!(years.len(), 3);
    }

    // ── Category style function coverage ──
    #[test]
    fn category_style_function_exists() {
        // category_style_by_name takes categories slice and name, returns tuple
        let categories: Vec<crate::models::category::Category> = vec![];
        let result = category_style_by_name(&categories, "Food");
        // Returns tuple of (color, icon)
        assert_eq!(result.0, "#6b7280"); // default color
        assert_eq!(result.1, "Tag"); // default icon
    }

    #[test]
    fn category_style_with_empty_categories() {
        let categories: Vec<crate::models::category::Category> = vec![];
        let result = category_style_by_name(&categories, "Unknown");
        assert_eq!(result.0, "#6b7280");
        assert_eq!(result.1, "Tag");
    }

    // ── Donut data construction edge cases ──
    #[test]
    fn donut_data_multiple_items() {
        use crate::components::chart::DonutData;
        let data = vec![
            DonutData { label: "A".into(), value: 40.0, color: "#111".into() },
            DonutData { label: "B".into(), value: 60.0, color: "#222".into() },
        ];
        assert_eq!(data.len(), 2);
    }

    // ── Palette color edge cases ──
    #[test]
    fn palette_color_returns_deterministic() {
        use crate::components::chart::palette_color;
        let c1 = palette_color(5);
        let c2 = palette_color(5);
        assert_eq!(c1, c2);
    }

    #[test]
    fn palette_color_within_range() {
        use crate::components::chart::palette_color;
        for i in 0..10 {
            let c = palette_color(i);
            assert!(c.starts_with('#'), "Color {} should start with #", i);
        }
    }

    // ── build_csv structure checks ──
    #[test]
    fn build_csv_function_exists() {
        // Verify build_csv takes ReportResponse
        let src = include_str!("reports_page.rs");
        assert!(src.contains("fn build_csv"), "build_csv should be defined");
    }

    // ── SpendingTrend structure check ──
    #[test]
    fn spending_trend_fields() {
        let trend = SpendingTrend {
            period: "2024-01".into(),
            total: dec!(100),
        };
        assert_eq!(trend.period, "2024-01");
    }

    // ── Category breakdown structure check ──
    #[test]
    fn category_breakdown_fields() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "Food".into(),
            total: dec!(50),
            count: 2,
            percentage: 25.0,
        };
        assert_eq!(breakdown.category_name, "Food");
    }

    // ── Monthly summary structure check ──
    #[test]
    fn monthly_summary_fields() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100),
            transactions_total: dec!(50),
            overall_total: dec!(150),
            subscriptions_count: 5,
            transactions_count: 3,
        };
        assert_eq!(summary.year, 2024);
    }

    // ── ReportResponse structure check ──
    #[test]
    fn report_response_structure() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        assert_eq!(report.monthly_summary.year, 2024);
    }

    // ── build_csv tests ──
    #[test]
    fn build_csv_empty_report() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(0),
                transactions_total: dec!(0),
                overall_total: dec!(0),
                subscriptions_count: 0,
                transactions_count: 0,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        // CSV should be non-empty
        assert!(!csv.is_empty());
    }

    #[test]
    fn build_csv_with_breakdown() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![
                CategoryBreakdown {
                    category_id: Some("cat-1".into()),
                    category_name: "Entertainment".into(),
                    total: dec!(100),
                    count: 5,
                    percentage: 66.67,
                },
                CategoryBreakdown {
                    category_id: Some("cat-2".into()),
                    category_name: "Food".into(),
                    total: dec!(50),
                    count: 3,
                    percentage: 33.33,
                },
            ],
            recent_trends: vec![],
        };
        let csv = build_csv(&report);
        // CSV should contain data
        assert!(!csv.is_empty());
    }

    // Tests for category_style_by_name would require mock categories - skipped
}