//! Monthly reports: summary, category split and recent trends.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::theme;
use nsh_core::canvas::{CanvasTextAlign, LineJoin};
#[cfg(not(target_arch = "wasm32"))]
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use std::f32::consts::TAU;

use crate::api::{self, ReportsData};
use crate::auth::AuthSession;
use crate::components::compact_select::compact_select;
use crate::components::dropdown_menu::{MenuAction, dropdown_menu};
use crate::components::page_header::page_header as shared_page_header;
use crate::format::format_currency;
use crate::models::{Category, ReportResponse};
use crate::theme as design;
use crate::ui;

const TITLE: &str = "Reports";
const SUBTITLE: &str = "Analyze your spending patterns";

pub fn reports(auth: AuthSession, compact: Signal<bool>) -> AnyElement {
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
    let month_choice = signal(today.month().to_string());
    let year_choice = signal(today.year().to_string());
    effect(move || {
        if let Ok(value) = month_choice.get().parse::<u32>() {
            month.set(value);
        }
    });
    effect(move || {
        if let Ok(value) = year_choice.get().parse::<i32>() {
            year.set(value);
        }
    });
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
        .pt(move || {
            if crate::ui::compact_active() {
                px(20.0)
            } else {
                px(0.0)
            }
        })
        .child(shared_page_header(
            TITLE,
            SUBTITLE,
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .justify_start()
                .items_center()
                .child(
                    when(
                        move || !compact.get(),
                        move || view().flex_grow(1.0).basis(px(0.0)).into_element(),
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    view().w(px(144.0)).mr(px(8.0)).child(compact_select(
                        month_choice,
                        (1..=12)
                            .map(|month| (month.to_string(), month_name(month).to_string()))
                            .collect(),
                        144.0,
                    )),
                )
                .child(
                    view().w(px(98.0)).mr(px(10.0)).child(compact_select(
                        year_choice,
                        (today.year() - 2..=today.year())
                            .map(|year| (year.to_string(), year.to_string()))
                            .collect(),
                        96.0,
                    )),
                )
                .child(export_menu(data, &currency))
                .into_element(),
            compact,
            false,
            400.0,
        ))
        .child(when(loading, || loading_card()).otherwise(move || {
            let currency = render_currency.clone();
            when(has_error, move || {
                error_card(&data.error().unwrap_or_default(), data)
            })
            .otherwise(move || content(data, currency.clone(), compact))
        }))
        .into_element()
}

fn content(
    data: Resource<ReportsData, String>,
    currency: String,
    compact: Signal<bool>,
) -> AnyElement {
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
        .gap(px(24.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .gap(px(13.0))
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
                    "Avg / Month",
                    asset!("icons/trending.svg"),
                    average,
                    move || format!("over {} months", report().recent_trends.len()),
                )),
        )
        .child({
            let charts = if compact.get() {
                view().w_full().flex_col().gap(px(16.0))
            } else {
                view().w_full().flex_row().items_start().gap(px(24.0))
            };
            charts
                .child(trend_chart(data, &trend_currency))
                .child(category_donut(data, &category_currency, compact))
        })
        .child(category_table(data, &currency, compact))
        .into_element()
}

fn export_menu(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let csv_currency = currency.to_string();
    let pdf_currency = currency.to_string();
    let csv_data = data;
    let csv = move || {
        let Some(snapshot) = csv_data.get() else {
            return;
        };
        let csv = build_csv(&snapshot.report, &csv_currency);
        #[cfg(target_arch = "wasm32")]
        {
            let _ = crate::web_files::download_text("recurrency-report.csv", "text/csv", &csv);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    };
    let pdf = move || {
        let Some(snapshot) = data.get() else {
            return;
        };
        let pdf = build_pdf(&snapshot.report, &pdf_currency);
        #[cfg(target_arch = "wasm32")]
        {
            let _ =
                crate::web_files::download_bytes("recurrency-report.pdf", "application/pdf", &pdf);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
        }
    };
    dropdown_menu(
        "Export",
        Some(asset!("icons/download.svg")),
        vec![
            MenuAction::new("Export CSV", csv),
            MenuAction::new("Export PDF", pdf),
        ],
    )
}

fn month_name(month: u32) -> &'static str {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    MONTHS
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("Month")
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
        .min_h(px(126.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border.opacity(0.4))
        .bg(move || theme().card.opacity(0.8))
        .p(px(24.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .pb(px(10.0))
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
            .line_height(px(32.0))
            .letter_spacing(px(-0.6))
            .font_family(design::MONO)
            .font_weight(FontWeight::BOLD)
            .text_color(move || theme().foreground),
        )
        .child(
            text(caption)
                .text_size(px(12.0))
                .line_height(px(16.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn category_donut(
    data: Resource<ReportsData, String>,
    currency: &str,
    compact: Signal<bool>,
) -> impl IntoElement {
    let _currency = currency.to_string();
    let hovered = signal(None::<usize>);
    view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(320.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border.opacity(0.4))
        .bg(move || theme().card.opacity(0.8))
        .p(move || if compact.get() { px(20.0) } else { px(24.0) })
        .min_h(px(314.0))
        .gap(px(16.0))
        .child(
            text("By Category")
                .text_size(px(18.0))
                .line_height(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .relative()
                .w_full()
                .items_center()
                .child(
                    canvas(move || {
                        let size = 180.0_f32;
                        let center = size / 2.0;
                        let radius = 72.0;
                        let mut painter = Painter::new()
                            .stroke_color(rgb(0x454759))
                            .line_width(25.2)
                            .begin_path()
                            .arc(center, center, radius, 0.0, TAU)
                            .stroke()
                            .line_width(21.6);
                        let categories = data.get().unwrap_or_default().report.category_breakdown;
                        let total: f64 =
                            categories.iter().map(|category| category.percentage).sum();
                        if categories.is_empty() || total <= 0.0 {
                            return painter;
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
                                .line_width(if hovered.get() == Some(index) {
                                    27.0
                                } else {
                                    21.6
                                })
                                .begin_path()
                                .arc(center, center, radius, start + 0.02, start + sweep - 0.02)
                                .stroke();
                            start += sweep;
                        }
                        painter
                    })
                    .h(px(180.0))
                    .w(px(180.0))
                    .on_mouse_move(move |event: &MouseEvent| {
                        let center = 90.0_f32;
                        let x = event.position.x.0 - center;
                        let y = event.position.y.0 - center;
                        let distance = (x * x + y * y).sqrt();
                        if !(55.0..=92.0).contains(&distance) {
                            hovered.set(None);
                            return;
                        }
                        let categories = data.get().unwrap_or_default().report.category_breakdown;
                        if categories.is_empty() {
                            hovered.set(None);
                            return;
                        }
                        if categories.len() == 1 {
                            hovered.set(Some(0));
                            return;
                        }
                        let total: f64 =
                            categories.iter().map(|category| category.percentage).sum();
                        if total <= 0.0 {
                            hovered.set(None);
                            return;
                        }
                        let angle = (y.atan2(x) + TAU / 4.0).rem_euclid(TAU);
                        let mut start = 0.0_f32;
                        for (index, category) in categories.iter().enumerate() {
                            let sweep = (category.percentage / total) as f32 * TAU;
                            if angle >= start && angle < start + sweep {
                                hovered.set(Some(index));
                                return;
                            }
                            start += sweep;
                        }
                        hovered.set(None);
                    })
                    .on_mouse_leave(move |_: &MouseEvent| hovered.set(None)),
                )
                .child(when(
                    move || hovered.get().is_some(),
                    move || {
                        let categories = data.get().unwrap_or_default().report.category_breakdown;
                        let index = hovered.get().unwrap_or(0);
                        let label = categories.get(index).map_or_else(String::new, |category| {
                            format!("{} · {:.0}%", category.category_name, category.percentage)
                        });
                        view()
                            .absolute()
                            .inset_0()
                            .items_center()
                            .justify_center()
                            .pointer_events(PointerEvents::None)
                            .child(
                                view()
                                    .rounded(px(6.0))
                                    .bg(move || theme().popover)
                                    .border(px(1.0))
                                    .border_color(move || theme().border)
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .child(
                                        text(label)
                                            .text_size(px(11.0))
                                            .text_color(move || theme().foreground),
                                    ),
                            )
                            .into_element()
                    },
                )),
        )
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(if compact.get() {
                    SharedString::from("repeat(1, minmax(0, 1fr))")
                } else {
                    SharedString::from("repeat(2, minmax(0, 1fr))")
                })
                .gap(px(6.0))
                .child(each(
                    move || {
                        data.get()
                            .unwrap_or_default()
                            .report
                            .category_breakdown
                            .into_iter()
                            .enumerate()
                            .collect::<Vec<_>>()
                    },
                    |(index, category)| format!("{index}-{}", category.category_name),
                    move |item, _| {
                        let (index, category) = item.get();
                        let color = category_color(data, &category.category_name);
                        let name = category.category_name.clone();
                        let percentage = format!("{:.0}%", category.percentage);
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .px(px(4.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(move || {
                                if hovered.get() == Some(index) {
                                    theme().muted.opacity(0.5)
                                } else {
                                    theme().card.opacity(0.0)
                                }
                            })
                            .opacity(move || {
                                if hovered.get().is_none() || hovered.get() == Some(index) {
                                    1.0_f32
                                } else {
                                    0.35_f32
                                }
                            })
                            .on_mouse_enter(move |_: &MouseEvent| hovered.set(Some(index)))
                            .on_mouse_leave(move |_: &MouseEvent| hovered.set(None))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .min_w(px(0.0))
                                    .flex_grow(1.0)
                                    .child(
                                        view()
                                            .h(px(10.0))
                                            .w(px(10.0))
                                            .rounded(px(999.0))
                                            .bg(move || rgb(parse_hex(&color))),
                                    )
                                    .child(
                                        image(asset!("icons/tag.svg"))
                                            .size(px(16.0))
                                            .tint_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text(name)
                                            .flex_grow(1.0)
                                            .min_w(px(0.0))
                                            .number_of_lines(1)
                                            .text_size(px(14.0))
                                            .line_height(px(20.0))
                                            .text_color(move || theme().muted_foreground),
                                    ),
                            )
                            .child(
                                text(percentage)
                                    .flex_shrink_0()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().foreground),
                            )
                    },
                )),
        )
}

fn category_table(
    data: Resource<ReportsData, String>,
    currency: &str,
    compact: Signal<bool>,
) -> impl IntoElement {
    let currency = currency.to_string();
    view()
        .w_full()
        .min_h(px(142.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border.opacity(0.4))
        .bg(move || theme().card.opacity(0.8))
        .p(move || if compact.get() { px(20.0) } else { px(24.0) })
        .gap(px(16.0))
        .child(
            text("Category Breakdown")
                .text_size(px(18.0))
                .line_height(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(view().flex_col().gap(px(16.0)).child(each(
            move || data.get().unwrap_or_default().report.category_breakdown,
            |category| category.category_name.clone(),
            move |category, _| {
                let category = category.get();
                let color = category_color(data, &category.category_name);
                let color_bg = color.clone();
                let color_dot = color.clone();
                let color_bar = color;
                let amount = format_currency(category.total, &currency);
                let count = category.count;
                let percentage = category.percentage.max(0.0) as f32;
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(10.0))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .min_w(px(0.0))
                                    .flex_grow(1.0)
                                    .child(
                                        view()
                                            .h(px(32.0))
                                            .w(px(32.0))
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(999.0))
                                            .bg(move || rgba((parse_hex(&color_bg) << 8) | 0x20))
                                            .child(
                                                view()
                                                    .h(px(16.0))
                                                    .w(px(16.0))
                                                    .rounded(px(999.0))
                                                    .bg(move || rgb(parse_hex(&color_dot))),
                                            ),
                                    )
                                    .child(
                                        image(asset!("icons/tag.svg"))
                                            .size(px(16.0))
                                            .tint_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text(category.category_name.clone())
                                            .flex_grow(1.0)
                                            .min_w(px(0.0))
                                            .number_of_lines(1)
                                            .text_size(px(16.0))
                                            .line_height(px(24.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || theme().foreground),
                                    ),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(16.0))
                                    .child(
                                        text(ui::count_label(count as usize, "item", "items"))
                                            .text_size(px(16.0))
                                            .line_height(px(24.0))
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text(move || {
                                            if crate::ui::privacy_active() {
                                                "••••".to_string()
                                            } else {
                                                amount.clone()
                                            }
                                        })
                                        .text_size(px(16.0))
                                        .line_height(px(24.0))
                                        .letter_spacing(px(-0.3))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().foreground),
                                    ),
                            ),
                    )
                    .child(
                        view()
                            .h(px(8.0))
                            .w_full()
                            .rounded(px(999.0))
                            .overflow_hidden()
                            .bg(move || theme().muted)
                            .child(
                                view()
                                    .h_full()
                                    .flex_grow(percentage.max(0.01))
                                    .bg(move || rgb(parse_hex(&color_bar))),
                            ),
                    )
                    .into_element()
            },
        )))
}

fn trend_label(period: &str) -> String {
    if period.contains('/') {
        return period.to_string();
    }
    let mut parts = period.split('-');
    let year = parts.next().unwrap_or_default();
    let month = parts.next().unwrap_or_default().trim_start_matches('0');
    let short_year = year.get(2..).unwrap_or(year);
    format!("{month}/{short_year}")
}

fn trend_chart(data: Resource<ReportsData, String>, currency: &str) -> impl IntoElement {
    let _currency = currency.to_string();
    let width = signal(0.0_f32);
    view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(320.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border.opacity(0.4))
        .bg(move || theme().card.opacity(0.8))
        .p(px(24.0))
        .min_h(px(314.0))
        .gap(px(16.0))
        .child(
            text("Spending Trends")
                .text_size(px(18.0))
                .line_height(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            canvas(move || {
                let mut trends = data.get().unwrap_or_default().report.recent_trends;
                trends.reverse();
                let w = width.get().max(1.0);
                let h: f32 = 220.0;
                let chart_w: f32 = 500.0;
                let offset = ((w - chart_w) / 2.0).max(0.0);
                let left = offset + 60.0;
                let right = offset + 16.0;
                let top: f32 = 16.0;
                let bottom: f32 = 32.0;
                let plot_w = (chart_w - 60.0 - 16.0).max(1.0);
                let plot_h: f32 = (h - top - bottom).max(1.0);
                let max_trend = trends
                    .iter()
                    .map(|trend| trend.total)
                    .max()
                    .unwrap_or(Decimal::ZERO);
                let max = max_trend.to_string().parse::<f32>().unwrap_or(0.0).max(1.0);
                let mut painter = Painter::new();
                for step in 0..=4 {
                    let value = max * step as f32 / 4.0;
                    let y = top + plot_h - value / max * plot_h;
                    painter = painter
                        .stroke_color(rgb(0x313244))
                        .line_width(1.0)
                        .begin_path()
                        .move_to(left, y)
                        .line_to(right + plot_w, y)
                        .stroke();
                    if !crate::ui::privacy_active() {
                        painter = painter
                            .fill_color(rgb(0x949AB3))
                            .font("10px Cabinet Grotesk")
                            .text_align(CanvasTextAlign::Right)
                            .fill_text(&format!("{value:.0}"), left - 6.0, y + 3.0);
                    }
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
                    .stroke_color(rgb(0x6366F1))
                    .line_width(2.5)
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
                    let radius = if trend.total == max_trend { 5.5 } else { 3.5 };
                    painter = painter
                        .fill_color(rgb(0x6366F1))
                        .begin_path()
                        .arc(cx, cy, radius, 0.0, TAU)
                        .fill()
                        .fill_color(rgb(0x949AB3))
                        .font("10px Cabinet Grotesk")
                        .text_align(CanvasTextAlign::Center)
                        .fill_text(&trend_label(&trend.period), cx, h - 13.0);
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
    if name == "Uncategorized" {
        return "#6b7280".to_string();
    }
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
    fn month_names_are_one_based_and_fall_back_safely() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(0), "January");
        assert_eq!(month_name(13), "Month");
    }
}
