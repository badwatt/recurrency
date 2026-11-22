//! Monthly reports: summary, category split and recent trends.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::canvas::{CanvasTextAlign, LineJoin};
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use std::f32::consts::TAU;

use crate::api::{self, ReportsData};
use crate::auth::AuthSession;
use crate::components::page_header::page_header as shared_page_header;
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
        .child(shared_page_header(
            TITLE,
            SUBTITLE,
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(navigation(year, month, today))
                .child(export_button(data, &currency))
                .child(export_pdf_button(data, &currency))
                .into_element(),
        ))
        .child(when(loading, || loading_card()).otherwise(move || {
            let currency = render_currency.clone();
            when(has_error, move || {
                error_card(&data.error().unwrap_or_default(), data)
            })
            .otherwise(move || content(data, currency.clone()))
        }))
        .into_element()
}

fn navigation(year: Signal<i32>, month: Signal<u32>, today: NaiveDate) -> impl IntoElement {
    view()
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
    let total = move || format_currency(report().monthly_summary.overall_total, &total_currency);
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
    let subscriptions_caption_currency = currency.clone();
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
                .flex_wrap()
                .gap(px(12.0))
                .child(summary_card(
                    "Total Spending",
                    asset!("icons/wallet.svg"),
                    total,
                    move || {
                        let summary = report().monthly_summary;
                        NaiveDate::from_ymd_opt(summary.year, summary.month, 1)
                            .map_or_else(String::new, |date| date.format("%B %Y").to_string())
                    },
                ))
                .child(summary_card(
                    "Subscriptions",
                    asset!("icons/card.svg"),
                    subscriptions,
                    move || {
                        let summary = report().monthly_summary;
                        format!(
                            "{} active · {} / year",
                            summary.subscriptions_count,
                            format_currency(
                                summary.subscriptions_total * Decimal::from(12),
                                &subscriptions_caption_currency
                            )
                        )
                    },
                ))
                .child(summary_card(
                    "One-time",
                    asset!("icons/receipt.svg"),
                    transactions,
                    move || {
                        format!(
                            "{} transactions",
                            report().monthly_summary.transactions_count
                        )
                    },
                ))
                .child(summary_card(
                    "Avg / month",
                    asset!("icons/trending.svg"),
                    average,
                    move || format!("over {} months", report().recent_trends.len()),
                )),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .gap(px(16.0))
                .child(trend_chart(data, &trend_currency))
                .child(category_donut(data, &category_currency)),
        )
        .child(category_table(data, &currency))
        .into_element()
}

fn export_button(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    button()
        .items_center()
        .justify_center()
        .h(px(38.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .on_press(move || {
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
        })
        .child(
            text("Export CSV")
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().foreground),
        )
}

fn export_pdf_button(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    button()
        .items_center()
        .justify_center()
        .h(px(38.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .on_press(move || {
            let Some(snapshot) = data.get() else {
                return;
            };
            let pdf = build_pdf(&snapshot.report, &currency);
            spawn(async move {
                if let Ok(Some(path)) = pick_save_path(
                    SaveDialog::new()
                        .file_name("recurrency-report.pdf")
                        .filter(FileFilter::new("PDF", ["pdf"])),
                )
                .await
                {
                    let _ = std::fs::write(path, pdf);
                }
            });
        })
        .child(
            text("Export PDF")
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().foreground),
        )
}

fn summary_card(
    label: &'static str,
    icon: ImageAsset,
    value: impl Fn() -> String + 'static,
    caption: impl Fn() -> String + 'static,
) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .min_w(px(200.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(18.0))
        .gap(px(4.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text(label)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    image(icon)
                        .size(px(16.0))
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .child(
            text(move || {
                if crate::ui::privacy_active() {
                    "••••".to_string()
                } else {
                    value()
                }
            })
            .text_size(px(24.0))
            .font_family(design::MONO)
            .font_weight(FontWeight::BOLD)
            .text_color(move || theme().foreground),
        )
        .child(
            text(caption)
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn category_donut(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let _currency = currency.to_string();
    view()
        .flex_grow(1.0)
        .min_w(px(320.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(12.0))
        .child(
            text("By Category")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view().w_full().items_center().child(
                canvas(move || {
                    let size = 240.0_f32;
                    let center = size / 2.0;
                    let radius = size * 0.34;
                    let mut painter = Painter::new().line_width(size * 0.11);
                    let categories = data.get().unwrap_or_default().report.category_breakdown;
                    if categories.is_empty() {
                        return painter
                            .stroke_color(rgb(0x454759))
                            .begin_path()
                            .arc(center, center, radius, 0.0, TAU)
                            .stroke();
                    }
                    let total: f64 = categories.iter().map(|category| category.percentage).sum();
                    if total <= 0.0 {
                        return painter
                            .stroke_color(rgb(0x454759))
                            .begin_path()
                            .arc(center, center, radius, 0.0, TAU)
                            .stroke();
                    }
                    if categories.len() == 1 {
                        let color = category_color(data, &categories[0].category_name);
                        return painter
                            .stroke_color(rgb(parse_hex(&color)))
                            .begin_path()
                            .arc(center, center, radius, 0.0, TAU)
                            .stroke();
                    }
                    let mut start = -TAU / 4.0;
                    for (index, category) in categories.iter().enumerate() {
                        let sweep = (category.percentage / total) as f32 * TAU;
                        let color = category_color(data, &category.category_name);
                        painter = painter
                            .stroke_color(rgb(parse_hex(&color)))
                            .begin_path()
                            .arc(center, center, radius, start + 0.02, start + sweep - 0.02)
                            .stroke();
                        start += sweep;
                        let _ = index;
                    }
                    painter
                })
                .h(px(240.0))
                .w(px(240.0)),
            ),
        )
        .child(view().flex_col().gap(px(6.0)).child(each(
            move || data.get().unwrap_or_default().report.category_breakdown,
            |category| category.category_name.clone(),
            move |category, _| {
                let category = category.get();
                let color = category_color(data, &category.category_name);
                let name = category.category_name.clone();
                let percentage = format!("{:.0}%", category.percentage);
                view()
                    .w_full()
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
                                    .bg(move || rgb(parse_hex(&color))),
                            )
                            .child(
                                text(name)
                                    .text_size(px(12.0))
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        text(percentage)
                            .text_size(px(12.0))
                            .font_family(design::MONO)
                            .text_color(move || theme().muted_foreground),
                    )
            },
        )))
}

fn category_table(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let currency = currency.to_string();
    view()
        .w_full()
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(10.0))
        .child(
            text("Category Breakdown")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .py(px(8.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .child(
                    text("CATEGORY")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .flex_grow(2.0),
                )
                .child(
                    text("ITEMS")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .flex_grow(1.0),
                )
                .child(
                    text("SHARE")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .flex_grow(1.0),
                )
                .child(
                    text("AMOUNT")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .flex_grow(1.0),
                ),
        )
        .child(view().flex_col().child(each(
            move || data.get().unwrap_or_default().report.category_breakdown,
            |category| category.category_name.clone(),
            move |category, _| {
                let category = category.get();
                let color = category_color(data, &category.category_name);
                let amount = format_currency(category.total, &currency);
                let name = category.category_name.clone();
                let count = category.count;
                let share = format!("{:.1}%", category.percentage);
                view()
                    .w_full()
                    .flex_row()
                    .items_center()
                    .py(px(9.0))
                    .border_b(px(1.0))
                    .border_color(move || theme().border)
                    .child(
                        view()
                            .flex_grow(2.0)
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                view()
                                    .h(px(8.0))
                                    .w(px(8.0))
                                    .rounded(px(999.0))
                                    .bg(move || rgb(parse_hex(&color))),
                            )
                            .child(
                                text(name)
                                    .text_size(px(12.0))
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        text(count.to_string())
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground)
                            .flex_grow(1.0),
                    )
                    .child(
                        text(share)
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground)
                            .flex_grow(1.0),
                    )
                    .child(
                        text(move || {
                            if crate::ui::privacy_active() {
                                "••••".to_string()
                            } else {
                                amount.clone()
                            }
                        })
                        .text_size(px(12.0))
                        .font_family(design::MONO)
                        .text_color(move || theme().foreground)
                        .flex_grow(1.0),
                    )
            },
        )))
}

fn trend_chart(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let _currency = currency.to_string();
    let width = signal(0.0_f32);
    view()
        .flex_grow(1.0)
        .min_w(px(320.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(12.0))
        .child(
            text("Spending Trends")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            canvas(move || {
                let mut trends = data.get().unwrap_or_default().report.recent_trends;
                trends.reverse();
                let w = width.get().max(1.0);
                let h: f32 = 220.0;
                let left = 34.0;
                let right = 8.0;
                let top: f32 = 10.0;
                let bottom: f32 = 28.0;
                let plot_w = (w - left - right).max(1.0);
                let plot_h: f32 = (h - top - bottom).max(1.0);
                let max = trends
                    .iter()
                    .map(|trend| trend.total)
                    .max()
                    .unwrap_or(Decimal::ZERO)
                    .to_string()
                    .parse::<f32>()
                    .unwrap_or(0.0)
                    .max(1.0);
                let mut painter = Painter::new();
                for step in 0..=4 {
                    let value = max * step as f32 / 4.0;
                    let y = top + plot_h - value / max * plot_h;
                    painter = painter
                        .stroke_color(rgb(0x313244))
                        .line_width(1.0)
                        .begin_path()
                        .move_to(left, y)
                        .line_to(w - right, y)
                        .stroke()
                        .fill_color(rgb(0x949AB3))
                        .font("10px Cabinet Grotesk")
                        .text_align(CanvasTextAlign::Right)
                        .fill_text(&format!("{value:.0}"), left - 6.0, y + 3.0);
                }
                if trends.is_empty() {
                    return painter;
                }
                let x = |index: usize| {
                    if trends.len() <= 1 {
                        left + plot_w / 2.0
                    } else {
                        left + index as f32 / (trends.len() - 1) as f32 * plot_w
                    }
                };
                let y = |total: Decimal| {
                    let value = total.to_string().parse::<f32>().unwrap_or(0.0);
                    top + plot_h - value / max * plot_h
                };
                painter = painter
                    .fill_color(rgba(0x6366f133))
                    .begin_path()
                    .move_to(x(0), top + plot_h);
                for (index, trend) in trends.iter().enumerate() {
                    painter = painter.line_to(x(index), y(trend.total));
                }
                painter = painter
                    .line_to(x(trends.len() - 1), top + plot_h)
                    .close_path()
                    .fill()
                    .stroke_color(rgb(0xB4BEFE))
                    .line_width(2.0)
                    .line_join(LineJoin::Round)
                    .begin_path()
                    .move_to(x(0), y(trends[0].total));
                for (index, trend) in trends.iter().enumerate().skip(1) {
                    painter = painter.line_to(x(index), y(trend.total));
                }
                painter = painter.stroke();
                for (index, trend) in trends.iter().enumerate() {
                    let cx = x(index);
                    let cy = y(trend.total);
                    painter = painter
                        .fill_color(rgb(0xB4BEFE))
                        .begin_path()
                        .arc(cx, cy, 3.0, 0.0, TAU)
                        .fill()
                        .fill_color(rgb(0x949AB3))
                        .font("10px Cabinet Grotesk")
                        .text_align(CanvasTextAlign::Center)
                        .fill_text(&trend.period, cx, h - 8.0);
                }
                painter
            })
            .w_full()
            .h(px(220.0))
            .on_layout(
                move |bounds: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
                    width.set(bounds.size.width.0);
                },
            ),
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

fn pdf_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn build_pdf(report: &ReportResponse, currency: &str) -> Vec<u8> {
    let summary = &report.monthly_summary;
    let period = NaiveDate::from_ymd_opt(summary.year, summary.month, 1)
        .map_or_else(String::new, |date| date.format("%B %Y").to_string());
    let mut lines = vec![
        ("Recurrency report".to_string(), 18.0),
        (period, 12.0),
        (String::new(), 10.0),
        (
            format!("Total spending: {} {}", summary.overall_total, currency),
            11.0,
        ),
        (
            format!(
                "Subscriptions: {} {} ({} active)",
                summary.subscriptions_total, currency, summary.subscriptions_count
            ),
            11.0,
        ),
        (
            format!(
                "One-time: {} {} ({} transactions)",
                summary.transactions_total, currency, summary.transactions_count
            ),
            11.0,
        ),
        (String::new(), 10.0),
        ("Category breakdown".to_string(), 13.0),
    ];
    for category in &report.category_breakdown {
        lines.push((
            format!(
                "{}: {} {} ({} items, {:.1}%)",
                category.category_name,
                category.total,
                currency,
                category.count,
                category.percentage
            ),
            10.0,
        ));
    }
    lines.push((String::new(), 10.0));
    lines.push(("Monthly trend".to_string(), 13.0));
    for trend in &report.recent_trends {
        lines.push((
            format!("{}: {} {}", trend.period, trend.total, currency),
            10.0,
        ));
    }
    let mut content = String::new();
    let mut y = 800.0;
    for (line, size) in lines {
        if y < 50.0 {
            break;
        }
        content.push_str(&format!(
            "BT /F1 {size} Tf 1 0 0 1 50 {y} Tm ({}) Tj ET\n",
            pdf_text(&line)
        ));
        y -= size + 7.0;
    }
    let objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
    }
    let xref = pdf.len();
    pdf.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        objects.len() + 1
    ));
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    pdf.into_bytes()
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
        .child(ui::action_button("Try again", false, move || {
            data.refetch()
        }))
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
