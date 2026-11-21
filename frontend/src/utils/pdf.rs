//! Minimal, dependency-free PDF writer for the monthly spending report.
//!
//! The report uses the standard Helvetica fonts with WinAnsi encoding, so the
//! generated file opens without embedding fonts and stays small. Layout code is
//! deterministic and pure so it can be covered by unit tests.

use crate::models::category::Category;
use crate::models::report::ReportResponse;
use crate::utils::color::{FALLBACK_COLOR, rgb};
use crate::utils::format::format_currency;
use std::fmt::Write;

/// A4 page width in PostScript points.
pub const PAGE_WIDTH: f32 = 595.28;
/// A4 page height in PostScript points.
pub const PAGE_HEIGHT: f32 = 841.89;

const MARGIN: f32 = 45.35;
const GAP: f32 = 9.0;
const CONTENT_TOP: f32 = 45.35;
const HEADER_BOTTOM: f32 = 96.0;
const CONTENT_START: f32 = 114.0;
const FOOTER_TOP: f32 = 54.0;
const BOTTOM_LIMIT: f32 = 68.0;
const TRACK_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN;

const INK: (f32, f32, f32) = (0.098, 0.125, 0.2);
const MUTED: (f32, f32, f32) = (0.41, 0.45, 0.53);
const ACCENT: (f32, f32, f32) = (0.388, 0.4, 0.945);
const ACCENT_SOFT: (f32, f32, f32) = (0.933, 0.941, 1.0);
const CARD: (f32, f32, f32) = (0.984, 0.988, 1.0);
const TRACK: (f32, f32, f32) = (0.933, 0.941, 0.969);
const RULE: (f32, f32, f32) = (0.91, 0.918, 0.949);
const ACCENT_BORDER: (f32, f32, f32) = (0.875, 0.890, 1.0);
const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);

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

/// Build a PDF report for the selected period.
pub fn build_report_pdf(
    report: &ReportResponse,
    currency: &str,
    categories: &[Category],
) -> Vec<u8> {
    let period = format!(
        "{} {}",
        month_name(report.monthly_summary.month),
        report.monthly_summary.year
    );
    let summary = &report.monthly_summary;
    let footer_right = format!(
        "{} active subscriptions · {} transactions",
        summary.subscriptions_count, summary.transactions_count
    );
    let mut pdf = Pdf::new(period, footer_right);
    pdf.summary(report, currency);
    pdf.categories(report, currency, categories);
    pdf.trends(report, currency);
    pdf.finish_page();
    pdf.into_bytes()
}

/// Encode text as Windows-1252 bytes for the built-in Helvetica fonts.
pub fn encode_winansi(text: &str) -> Vec<u8> {
    text.chars()
        .map(|character| match character {
            '\n' | '\r' | '\t' => b' ',
            character if character.is_ascii() => character as u8,
            '€' => 0x80,
            '‚' => 0x82,
            'ƒ' => 0x83,
            '„' => 0x84,
            '…' => 0x85,
            '†' => 0x86,
            '‡' => 0x87,
            'ˆ' => 0x88,
            '‰' => 0x89,
            'Š' => 0x8A,
            '‹' => 0x8B,
            'Œ' => 0x8C,
            'Ž' => 0x8E,
            '‘' => 0x91,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' => 0x95,
            '–' => 0x96,
            '—' => 0x97,
            '˜' => 0x98,
            '™' => 0x99,
            'š' => 0x9A,
            '›' => 0x9B,
            'œ' => 0x9C,
            'ž' => 0x9E,
            'Ÿ' => 0x9F,
            character if ('\u{00a0}'..='\u{00ff}').contains(&character) => character as u8,
            _ => b'?',
        })
        .collect()
}

/// Escape bytes for a PDF literal string.
pub fn escape_pdf_text(bytes: &[u8]) -> String {
    let mut escaped = String::with_capacity(bytes.len());
    for &byte in bytes {
        match byte {
            b'(' | b')' | b'\\' => {
                escaped.push('\\');
                escaped.push(byte as char);
            }
            0x20..=0x7e => escaped.push(byte as char),
            _ => {
                let _ = write!(escaped, "\\{byte:03o}");
            }
        }
    }
    escaped
}

/// Approximate text width for the built-in Helvetica fonts.
pub fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|character| match character {
            ' ' => 0.28,
            '0'..='9' => 0.556,
            'A'..='Z' => 0.667,
            'a'..='z' => 0.5,
            '.' | ',' | ':' | ';' | '\'' => 0.28,
            '%' => 0.89,
            _ => 0.556,
        })
        .sum::<f32>()
        * size
}

/// Width of a category bar, clamped to the track.
pub fn bar_width(percentage: f64, track_width: f32) -> f32 {
    (percentage / 100.0).clamp(0.0, 1.0) as f32 * track_width
}

/// English month name, falling back to `Month` for invalid values.
pub fn month_name(month: u32) -> &'static str {
    month
        .checked_sub(1)
        .and_then(|index| MONTHS.get(index as usize))
        .copied()
        .unwrap_or("Month")
}

/// Trim text to fit `max_width`, appending an ellipsis when needed.
pub fn truncate_to_width(text: &str, max_width: f32, size: f32) -> String {
    if estimate_text_width(text, size) <= max_width {
        return text.to_string();
    }
    let mut trimmed = String::new();
    let mut used = 0.0;
    for character in text.chars() {
        let width = estimate_text_width(&character.to_string(), size);
        if used + width > max_width - size * 0.5 {
            break;
        }
        used += width;
        trimmed.push(character);
    }
    trimmed.push('…');
    trimmed
}

type Color = (f32, f32, f32);

#[derive(Clone, Copy)]
struct PdfRect {
    x: f32,
    top: f32,
    width: f32,
    height: f32,
    radius: f32,
}

impl PdfRect {
    fn new(x: f32, top: f32, width: f32, height: f32, radius: f32) -> Self {
        Self {
            x,
            top,
            width,
            height,
            radius,
        }
    }
}

struct Pdf {
    pages: Vec<Vec<u8>>,
    y: f32,
    period: String,
    footer_right: String,
}

impl Pdf {
    fn new(period: String, footer_right: String) -> Self {
        let mut pdf = Self {
            pages: Vec::new(),
            y: CONTENT_TOP,
            period,
            footer_right,
        };
        pdf.start_page();
        pdf
    }

    fn start_page(&mut self) {
        self.pages.push(Vec::new());
        self.y = CONTENT_START;

        let mark_size = 31.5;
        self.rounded_rect_at(
            PdfRect::new(MARGIN, CONTENT_TOP, mark_size, mark_size, 9.0),
            Some(ACCENT),
            None,
        );
        let letter_size = 16.5;
        let letter_x = MARGIN + (mark_size - estimate_text_width("R", letter_size)) / 2.0;
        self.text_at(
            letter_x,
            CONTENT_TOP + (mark_size - letter_size) / 2.0 - 0.5,
            letter_size,
            true,
            WHITE,
            "R",
        );
        let brand_x = MARGIN + mark_size + GAP;
        self.text_at(brand_x, CONTENT_TOP - 0.5, 21.0, true, INK, "Recurrency");
        self.text_at(
            brand_x,
            CONTENT_TOP + 24.0,
            9.75,
            false,
            MUTED,
            "Monthly spending report",
        );
        self.right_text_at("Report period", CONTENT_TOP + 1.0, 9.75, false, MUTED);
        let period = self.period.clone();
        self.right_text_at(&period, CONTENT_TOP + 18.0, 13.5, true, INK);
        self.line_at(
            MARGIN,
            HEADER_BOTTOM,
            PAGE_WIDTH - MARGIN,
            HEADER_BOTTOM,
            RULE,
        );
    }

    fn finish_page(&mut self) {
        self.line_at(
            MARGIN,
            PAGE_HEIGHT - FOOTER_TOP,
            PAGE_WIDTH - MARGIN,
            PAGE_HEIGHT - FOOTER_TOP,
            RULE,
        );
        self.text_at(
            MARGIN,
            PAGE_HEIGHT - FOOTER_TOP + 8.0,
            7.5,
            false,
            MUTED,
            &format!("Generated by Recurrency {}", env!("CARGO_PKG_VERSION")),
        );
        let footer_right = self.footer_right.clone();
        self.right_text_at(
            &footer_right,
            PAGE_HEIGHT - FOOTER_TOP + 8.0,
            7.5,
            false,
            MUTED,
        );
    }

    fn ensure_space(&mut self, needed: f32) {
        if self.y + needed > PAGE_HEIGHT - BOTTOM_LIMIT {
            self.finish_page();
            self.start_page();
        }
    }

    fn summary(&mut self, report: &ReportResponse, currency: &str) {
        let summary = &report.monthly_summary;
        let cards = [
            (
                "Total spending",
                format_currency(summary.overall_total, currency),
            ),
            (
                "Subscriptions",
                format_currency(summary.subscriptions_total, currency),
            ),
            (
                "One-time",
                format_currency(summary.transactions_total, currency),
            ),
        ];
        let card_width = (PAGE_WIDTH - 2.0 * MARGIN - 2.0 * GAP) / 3.0;
        let card_height = 54.0;
        for (index, (label, value)) in cards.iter().enumerate() {
            let x = MARGIN + index as f32 * (card_width + GAP);
            let background = if index == 0 { ACCENT_SOFT } else { CARD };
            let border = if index == 0 { ACCENT_BORDER } else { RULE };
            self.rounded_rect_at(
                PdfRect::new(x, self.y, card_width, card_height, 9.0),
                Some(background),
                Some((border, 0.8)),
            );
            self.text_at(
                x + 12.0,
                self.y + 12.0,
                7.5,
                true,
                MUTED,
                &label.to_uppercase(),
            );
            self.text_at(x + 12.0, self.y + 30.0, 16.5, true, INK, value);
        }
        self.y += card_height + 21.0;
    }

    fn categories(&mut self, report: &ReportResponse, currency: &str, categories: &[Category]) {
        self.heading("By category");
        if report.category_breakdown.is_empty() {
            self.text_at(
                MARGIN,
                self.y,
                9.5,
                false,
                MUTED,
                "No category spending for this period.",
            );
            self.y += 24.0;
            return;
        }
        for category in &report.category_breakdown {
            self.ensure_space(58.0);
            let color = category_color(categories, &category.category_name);
            let amount = format_currency(category.total, currency);
            let label_width = PAGE_WIDTH - 2.0 * MARGIN - estimate_text_width(&amount, 9.75) - 28.0;
            let label = truncate_to_width(&category.category_name, label_width, 9.75);
            self.circle_at(MARGIN, self.y + 2.0, 3.4, color_f32(color));
            self.text_at(MARGIN + 12.0, self.y, 9.75, false, INK, &label);
            self.right_text_at(&amount, self.y, 9.75, true, INK);
            self.y += 15.0;
            self.rounded_rect_at(
                PdfRect::new(MARGIN, self.y, TRACK_WIDTH, 5.25, 2.6),
                Some(TRACK),
                None,
            );
            let width = bar_width(category.percentage, TRACK_WIDTH);
            if width > 0.0 {
                self.rounded_rect_at(
                    PdfRect::new(MARGIN, self.y, width, 5.25, 2.6),
                    Some(color_f32(color)),
                    None,
                );
            }
            self.y += 8.25;
            self.text_at(
                MARGIN,
                self.y,
                8.25,
                false,
                MUTED,
                &format!(
                    "{} items · {:.1}% of spending",
                    category.count,
                    category.percentage.clamp(0.0, 100.0)
                ),
            );
            self.y += 19.0;
        }
    }

    fn trends(&mut self, report: &ReportResponse, currency: &str) {
        self.heading("Spending trends");
        if report.recent_trends.is_empty() {
            self.text_at(
                MARGIN,
                self.y,
                9.5,
                false,
                MUTED,
                "No spending trends for this period.",
            );
            self.y += 24.0;
            return;
        }
        self.text_at(MARGIN, self.y, 7.5, true, MUTED, "PERIOD");
        self.right_text_at("TOTAL", self.y, 7.5, true, MUTED);
        self.y += 14.0;
        self.line_at(MARGIN, self.y, PAGE_WIDTH - MARGIN, self.y, RULE);
        for trend in &report.recent_trends {
            self.ensure_space(24.0);
            self.text_at(MARGIN, self.y + 6.0, 9.75, false, INK, &trend.period);
            let total = format_currency(trend.total, currency);
            self.right_text_at(&total, self.y + 6.0, 9.75, false, INK);
            self.y += 24.0;
            self.line_at(MARGIN, self.y, PAGE_WIDTH - MARGIN, self.y, RULE);
        }
        self.y += 8.0;
    }

    fn heading(&mut self, text: &str) {
        self.ensure_space(40.0);
        self.text_at(MARGIN, self.y, 13.0, true, INK, text);
        self.y += 26.0;
    }

    fn text_at(
        &mut self,
        x: f32,
        top: f32,
        size: f32,
        bold: bool,
        color: (f32, f32, f32),
        text: &str,
    ) {
        if let Some(page) = self.pages.last_mut() {
            let font = if bold { "F2" } else { "F1" };
            let baseline = PAGE_HEIGHT - top - size;
            let escaped = escape_pdf_text(&encode_winansi(text));
            let command = format!(
                "BT /{font} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg 1 0 0 1 {x:.2} {baseline:.2} Tm ({escaped}) Tj ET\n",
                r = color.0,
                g = color.1,
                b = color.2,
            );
            page.extend_from_slice(command.as_bytes());
        }
    }

    fn right_text_at(
        &mut self,
        text: &str,
        top: f32,
        size: f32,
        bold: bool,
        color: (f32, f32, f32),
    ) {
        let x = PAGE_WIDTH - MARGIN - estimate_text_width(text, size);
        self.text_at(x, top, size, bold, color, text);
    }

    fn rounded_rect_at(
        &mut self,
        rect: PdfRect,
        fill: Option<Color>,
        stroke: Option<(Color, f32)>,
    ) {
        let Some(page) = self.pages.last_mut() else {
            return;
        };
        let PdfRect {
            x,
            top,
            width,
            height,
            radius,
        } = rect;
        let bottom = PAGE_HEIGHT - top - height;
        let radius = radius.min(width / 2.0).min(height / 2.0);
        let curve = radius * 0.552_284_8;
        let mut command = String::from("q\n");
        if let Some((r, g, b)) = fill {
            let _ = writeln!(command, "{r:.3} {g:.3} {b:.3} rg");
        }
        if let Some(((r, g, b), line_width)) = stroke {
            let _ = writeln!(command, "{r:.3} {g:.3} {b:.3} RG {line_width:.2} w");
        }
        let _ = write!(
            command,
            "{x1:.2} {y:.2} m {x2:.2} {y:.2} l \
             {x3:.2} {y:.2} {x4:.2} {y3:.2} {x4:.2} {y4:.2} c \
             {x4:.2} {y5:.2} l \
             {x4:.2} {y6:.2} {x3:.2} {y7:.2} {x2:.2} {y7:.2} c \
             {x1:.2} {y7:.2} l \
             {x0:.2} {y7:.2} {x:.2} {y6:.2} {x:.2} {y5:.2} c \
             {x:.2} {y4:.2} l \
             {x:.2} {y3:.2} {x0:.2} {y:.2} {x1:.2} {y:.2} c h ",
            x = x,
            y = bottom,
            x0 = x + radius - curve,
            x1 = x + radius,
            x2 = x + width - radius,
            x3 = x + width - radius + curve,
            x4 = x + width,
            y3 = bottom + radius - curve,
            y4 = bottom + radius,
            y5 = bottom + height - radius,
            y6 = bottom + height - radius + curve,
            y7 = bottom + height,
        );
        if fill.is_some() && stroke.is_some() {
            command.push_str("B\nQ\n");
        } else if fill.is_some() {
            command.push_str("f\nQ\n");
        } else {
            command.push_str("S\nQ\n");
        }
        page.extend_from_slice(command.as_bytes());
    }

    fn circle_at(&mut self, left: f32, top: f32, radius: f32, color: (f32, f32, f32)) {
        let Some(page) = self.pages.last_mut() else {
            return;
        };
        let cx = left + radius;
        let cy = PAGE_HEIGHT - top - radius;
        let k = radius * 0.552_284_8;
        let command = format!(
            "q {r:.3} {g:.3} {b:.3} rg \
             {cx:.2} {bottom:.2} m \
             {cxk:.2} {bottom:.2} {right:.2} {cyk:.2} {right:.2} {cy:.2} c \
             {right:.2} {cyk2:.2} {cxk:.2} {top_y:.2} {cx:.2} {top_y:.2} c \
             {cxk3:.2} {top_y:.2} {left_edge:.2} {cyk2:.2} {left_edge:.2} {cy:.2} c \
             {left_edge:.2} {cyk:.2} {cxk3:.2} {bottom:.2} {cx:.2} {bottom:.2} c h f Q\n",
            r = color.0,
            g = color.1,
            b = color.2,
            cx = cx,
            cy = cy,
            bottom = cy - radius,
            top_y = cy + radius,
            left_edge = cx - radius,
            right = cx + radius,
            cxk = cx + k,
            cxk3 = cx - k,
            cyk = cy - k,
            cyk2 = cy + k,
        );
        page.extend_from_slice(command.as_bytes());
    }

    fn line_at(&mut self, x1: f32, top1: f32, x2: f32, top2: f32, color: (f32, f32, f32)) {
        if let Some(page) = self.pages.last_mut() {
            let y1 = PAGE_HEIGHT - top1;
            let y2 = PAGE_HEIGHT - top2;
            let command = format!(
                "q {r:.3} {g:.3} {b:.3} RG 0.6 w {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S Q\n",
                r = color.0,
                g = color.1,
                b = color.2,
            );
            page.extend_from_slice(command.as_bytes());
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        let page_count = self.pages.len();
        let font_regular = 3 + 2 * page_count;
        let font_bold = font_regular + 1;
        let mut objects: Vec<Vec<u8>> = Vec::with_capacity(4 + 2 * page_count);

        objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
        let kids = (0..page_count)
            .map(|index| format!("{} 0 R", 3 + 2 * index))
            .collect::<Vec<_>>()
            .join(" ");
        objects.push(format!("<< /Type /Pages /Kids [{kids}] /Count {page_count} >>").into_bytes());

        for (index, content) in self.pages.iter().enumerate() {
            let page_id = 3 + 2 * index;
            let content_id = page_id + 1;
            objects.push(
                format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.2} {PAGE_HEIGHT:.2}] /Resources << /Font << /F1 {font_regular} 0 R /F2 {font_bold} 0 R >> >> /Contents {content_id} 0 R >>"
                )
                .into_bytes(),
            );
            let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
            stream.extend_from_slice(content);
            stream.extend_from_slice(b"endstream");
            objects.push(stream);
        }

        objects.push(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        );
        objects.push(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        );

        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::with_capacity(objects.len());
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            pdf.extend_from_slice(object);
            pdf.extend_from_slice(b"\nendobj\n");
        }
        let xref_start = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }
}

fn category_color(categories: &[Category], name: &str) -> (u8, u8, u8) {
    categories
        .iter()
        .find(|category| category.name == name)
        .and_then(|category| category.color.as_deref())
        .and_then(rgb)
        .or_else(|| rgb(FALLBACK_COLOR))
        .unwrap_or((255, 255, 255))
}

fn color_f32(color: (u8, u8, u8)) -> (f32, f32, f32) {
    (
        f32::from(color.0) / 255.0,
        f32::from(color.1) / 255.0,
        f32::from(color.2) / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::report::{CategoryBreakdown, MonthlySummary, SpendingTrend};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn category(name: &str, color: &str) -> Category {
        Category {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            name: name.to_string(),
            icon: Some("Tag".to_string()),
            color: Some(color.to_string()),
            is_system: false,
            is_default: false,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    fn report(categories: usize) -> ReportResponse {
        let breakdown = (0..categories)
            .map(|index| CategoryBreakdown {
                category_id: None,
                category_name: format!("Category {index}"),
                total: dec!(10),
                count: 1,
                percentage: 100.0 / categories.max(1) as f64,
            })
            .collect();
        ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2026,
                month: 11,
                subscriptions_total: dec!(120.50),
                transactions_total: dec!(30),
                overall_total: dec!(150.50),
                subscriptions_count: 7,
                transactions_count: 4,
            },
            category_breakdown: breakdown,
            recent_trends: vec![
                SpendingTrend {
                    period: "2026-10".to_string(),
                    total: dec!(140),
                },
                SpendingTrend {
                    period: "2026-11".to_string(),
                    total: dec!(150.50),
                },
            ],
        }
    }

    fn xref_offsets(pdf: &[u8]) -> Vec<usize> {
        let marker = b"xref\n0 ";
        let position = pdf
            .windows(marker.len())
            .rposition(|window| window == marker)
            .expect("pdf has an xref table");
        let table = &pdf[position + marker.len()..];
        let newline = table.iter().position(|byte| *byte == b'\n').unwrap();
        let count: usize = std::str::from_utf8(&table[..newline])
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let body = &table[newline + 1..];
        let mut lines = body.split(|byte| *byte == b'\n');
        lines.next(); // free object
        lines
            .take(count - 1)
            .map(|line| {
                std::str::from_utf8(&line[..10])
                    .unwrap()
                    .parse::<usize>()
                    .unwrap()
            })
            .collect()
    }

    fn page_count(pdf: &[u8]) -> usize {
        let marker = b"/Type /Pages";
        let position = pdf
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("pages object exists");
        let rest = &pdf[position..];
        let count_marker = b"/Count ";
        let count_position = rest
            .windows(count_marker.len())
            .position(|window| window == count_marker)
            .unwrap();
        let digits = &rest[count_position + count_marker.len()..];
        let end = digits
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .unwrap();
        std::str::from_utf8(&digits[..end])
            .unwrap()
            .parse()
            .unwrap()
    }

    #[test]
    fn report_pdf_has_header_trailer_and_pages() {
        let pdf = build_report_pdf(&report(3), "EUR", &[category("Food", "#22c55e")]);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        assert_eq!(page_count(&pdf), 1);
        assert!(pdf.windows(7).any(|window| window == b"/F1 5 0"));
    }

    #[test]
    fn xref_entries_point_to_their_objects() {
        let pdf = build_report_pdf(&report(3), "EUR", &[category("Food", "#22c55e")]);
        let offsets = xref_offsets(&pdf);
        assert!(!offsets.is_empty());
        for (index, offset) in offsets.iter().enumerate() {
            let expected = format!("{} 0 obj", index + 1);
            assert!(
                pdf[*offset..].starts_with(expected.as_bytes()),
                "object {} offset {} does not point at its header",
                index + 1,
                offset
            );
        }
    }

    #[test]
    fn report_includes_period_summary_and_categories() {
        let pdf = build_report_pdf(&report(2), "EUR", &[category("Food", "#22c55e")]);
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.contains("November 2026"));
        assert!(content.contains("Monthly spending report"));
        assert!(content.contains("TOTAL SPENDING"));
        assert!(content.contains("Category 0"));
        assert!(content.contains("Spending trends"));
        assert!(content.contains("Generated by Recurrency"));
    }

    #[test]
    fn long_reports_paginate() {
        let pdf = build_report_pdf(&report(120), "EUR", &[category("Food", "#22c55e")]);
        assert!(page_count(&pdf) > 1);
        assert!(String::from_utf8_lossy(&pdf).contains("Spending trends"));
    }

    #[test]
    fn empty_report_is_still_a_valid_pdf() {
        let mut report = report(0);
        report.recent_trends.clear();
        let pdf = build_report_pdf(&report, "EUR", &[]);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert_eq!(page_count(&pdf), 1);
        assert!(String::from_utf8_lossy(&pdf).contains("No spending trends"));
    }

    #[test]
    fn winansi_maps_currency_and_punctuation() {
        assert_eq!(encode_winansi("€"), vec![0x80]);
        assert_eq!(encode_winansi("£"), vec![0xA3]);
        assert_eq!(encode_winansi("—"), vec![0x97]);
        assert_eq!(encode_winansi("…"), vec![0x85]);
        assert_eq!(encode_winansi("é"), vec![0xE9]);
        assert_eq!(encode_winansi("→"), vec![b'?']);
        assert_eq!(encode_winansi("a\nb"), vec![b'a', b' ', b'b']);
    }

    #[test]
    fn pdf_strings_escape_specials_and_binary_bytes() {
        assert_eq!(escape_pdf_text(b"plain"), "plain");
        assert_eq!(escape_pdf_text(b"(a)\\b)"), "\\(a\\)\\\\b\\)");
        assert_eq!(escape_pdf_text(&[0x80]), "\\200");
    }

    #[test]
    fn text_width_grows_with_text_and_size() {
        let short = estimate_text_width("A", 10.0);
        let long = estimate_text_width("AAAA", 10.0);
        let large = estimate_text_width("A", 20.0);
        assert!(short > 0.0);
        assert!(long > short);
        assert!(large > short);
    }

    #[test]
    fn truncation_respects_the_available_width() {
        let text = "A very long category name";
        let truncated = truncate_to_width(text, 40.0, 10.0);
        assert!(truncated.len() < text.len());
        assert!(truncated.ends_with('…'));
        assert_eq!(truncate_to_width("Short", 500.0, 10.0), "Short");
    }

    #[test]
    fn bar_width_is_clamped_to_the_track() {
        assert_eq!(bar_width(50.0, 200.0), 100.0);
        assert_eq!(bar_width(-10.0, 200.0), 0.0);
        assert_eq!(bar_width(150.0, 200.0), 200.0);
    }

    #[test]
    fn month_names_cover_the_year() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(0), "Month");
        assert_eq!(month_name(13), "Month");
    }

    #[test]
    fn invalid_category_colors_fall_back_without_breaking_the_pdf() {
        let pdf = build_report_pdf(&report(1), "EUR", &[category("Category 0", "not-a-color")]);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("Category 0"));
    }
}
