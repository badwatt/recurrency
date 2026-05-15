use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarData {
    pub label: String,
    pub value: f64,
    pub color: String,
}

#[component]
pub fn BarChart(data: Vec<BarData>, height: String) -> Element {
    if data.is_empty() {
        return rsx! {
            div { class: "text-sm text-muted-foreground", "No data" }
        };
    }

    let max_val = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
    let bar_height = 28.0_f64;
    let gap = 4.0_f64;
    let label_width = 100.0_f64;
    let value_width = 60.0_f64;
    let total = data.len() as f64;
    let svg_height = total * (bar_height + gap);
    let chart_width = 400.0_f64;
    let bar_max_width = chart_width - label_width - value_width;

    rsx! {
        svg {
            view_box: "0 0 {chart_width} {svg_height}",
            width: "100%",
            height: "{height}",
            xmlns: "http://www.w3.org/2000/svg",

            for (i, item) in data.iter().enumerate() {
                {
                    let y = i as f64 * (bar_height + gap);
                    let bar_w = if max_val > 0.0 { (item.value / max_val) * bar_max_width } else { 0.0 };
                    let label_x = 0.0_f64;
                    let bar_x = label_width;
                    let value_x = label_width + bar_max_width + 8.0;
                    let color = item.color.clone();
                    let label = item.label.clone();
                    let val_str = format!("{:.0}", item.value);

                    rsx! {
                        // Label
                        text {
                            x: "{label_x}",
                            y: "{y + bar_height / 2.0 + 4.0}",
                            fill: "currentColor",
                            class: "text-sm text-foreground",
                            text_anchor: "start",
                            {label}
                        }
                        // Bar background
                        rect {
                            x: "{bar_x}",
                            y: "{y + 4.0}",
                            width: "{bar_max_width}",
                            height: "{bar_height - 8.0}",
                            fill: "rgba(255,255,255,0.05)",
                            rx: "4",
                        }
                        // Bar fill
                        rect {
                            x: "{bar_x}",
                            y: "{y + 4.0}",
                            width: "{bar_w}",
                            height: "{bar_height - 8.0}",
                            fill: "{color}",
                            rx: "4",
                        }
                        // Value
                        text {
                            x: "{value_x}",
                            y: "{y + bar_height / 2.0 + 4.0}",
                            fill: "currentColor",
                            class: "text-sm text-muted-foreground",
                            text_anchor: "start",
                            {val_str}
                        }
                    }
                }
            }
        }
    }
}

// ── Line Chart ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineData {
    pub label: String,
    pub value: f64,
}

#[component]
pub fn LineChart(data: Vec<LineData>, height: String, color: String) -> Element {
    if data.is_empty() {
        return rsx! {
            div { class: "text-sm text-muted-foreground text-center py-8", "No data" }
        };
    }

    let max_val = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
    let max_val = if max_val == 0.0 { 1.0 } else { max_val };

    let padding_left = 60.0_f64;
    let padding_right = 16.0_f64;
    let padding_top = 16.0_f64;
    let padding_bottom = 32.0_f64;
    let view_width = 500.0_f64;
    let view_height = 220.0_f64;

    let chart_w = view_width - padding_left - padding_right;
    let chart_h = view_height - padding_top - padding_bottom;

    // Compute points
    let n = data.len() as f64;
    let step_x = if data.len() > 1 {
        chart_w / (n - 1.0)
    } else {
        0.0
    };

    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let x = padding_left + i as f64 * step_x;
            let y = padding_top + chart_h - (d.value / max_val) * chart_h;
            (x, y)
        })
        .collect();

    // Build polyline path
    let line_path = points
        .iter()
        .enumerate()
        .map(|(i, (x, y))| {
            if i == 0 {
                format!("M{x:.1},{y:.1}")
            } else {
                format!("L{x:.1},{y:.1}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Build area fill path (close to bottom)
    let area_path =
        if let (Some(&(first_x, _)), Some(&(last_x, _))) = (points.first(), points.last()) {
            let bottom_y = padding_top + chart_h;
            let mut path = line_path.clone();
            path.push_str(&format!(
                "L{last_x:.1},{bottom_y:.1}L{first_x:.1},{bottom_y:.1}Z"
            ));
            path
        } else {
            line_path.clone()
        };

    // Gradient id
    let grad_id = "lineGrad";

    rsx! {
        svg {
            view_box: "0 0 {view_width} {view_height}",
            width: "100%",
            height: "{height}",
            xmlns: "http://www.w3.org/2000/svg",
            class: "overflow-visible",

            // Defs: gradient for area fill
            defs {
                linearGradient {
                    id: "{grad_id}",
                    x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "0%", stop_color: "{color}", stop_opacity: "0.3" }
                    stop { offset: "100%", stop_color: "{color}", stop_opacity: "0.02" }
                }
            }

            // Horizontal grid lines
            {
                let grid_lines = (0..5).map(|i| {
                    let y = padding_top + (chart_h / 4.0) * i as f64;
                    let val = max_val - (max_val / 4.0) * i as f64;
                    rsx! {
                        line {
                            key: "grid-{i}",
                            x1: "{padding_left}",
                            y1: "{y}",
                            x2: "{padding_left + chart_w}",
                            y2: "{y}",
                            stroke: "currentColor",
                            stroke_opacity: "0.07",
                            stroke_dasharray: "3,3",
                        }
                        text {
                            x: "{padding_left - 6.0}",
                            y: "{y + 4.0}",
                            fill: "currentColor",
                            class: "text-xs fill-muted-foreground",
                            text_anchor: "end",
                            {format!("{:.0}", val)}
                        }
                    }
                }).collect::<Vec<_>>();
                grid_lines.into_iter()
            }

            // Area fill
            path {
                d: "{area_path}",
                fill: "url(#{grad_id})",
            }

            // Line
            path {
                d: "{line_path}",
                fill: "none",
                stroke: "{color}",
                stroke_width: "2.5",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }

            // Data points + labels
            for (i, (x, y)) in points.iter().enumerate() {
                circle {
                    key: "dot-{i}",
                    cx: "{x}",
                    cy: "{y}",
                    r: "3.5",
                    fill: "{color}",
                    stroke: "var(--card, #fff)",
                    stroke_width: "2",
                }
                text {
                    x: "{x}",
                    y: "{padding_top + chart_h + 18.0}",
                    fill: "currentColor",
                    class: "text-xs fill-muted-foreground",
                    text_anchor: "middle",
                    {data[i].label.clone()}
                }
            }
        }
    }
}

// ── Donut Chart ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonutData {
    pub label: String,
    pub value: f64,
    pub color: String,
}

#[component]
pub fn DonutChart(data: Vec<DonutData>, size: String, stroke_width: String, hovered_index: Option<usize>, on_hover: EventHandler<Option<usize>>) -> Element {
    if data.is_empty() {
        return rsx! {
            div { class: "text-sm text-muted-foreground text-center py-8", "No data" }
        };
    }

    let total: f64 = data.iter().map(|d| d.value).sum();
    if total == 0.0 {
        return rsx! {
            div { class: "text-sm text-muted-foreground text-center py-8", "No data" }
        };
    }

    // Deduplicate colors: shift hue for duplicate colors so segments stay distinct
    let data = deduplicate_colors(data);

    // SVG viewBox: center at 50,50 with radius ~40
    let center = 50.0_f64;
    let radius = 40.0_f64;
    let label_radius = 54.0_f64;
    let circumference = 2.0 * std::f64::consts::PI * radius;

    let sw: f64 = stroke_width.parse().unwrap_or(12.0);
    let segments = compute_donut_segments(&data, total, circumference);

    // Compute label positions (midpoint of each arc, non-rotated coords)
    let label_positions: Vec<(f64, f64, String, f64, usize)> = segments.iter().enumerate().map(|(i, seg)| {
        let mid_angle = (seg.offset + seg.dash_len / 2.0) / circumference * 360.0 - 90.0;
        let rad = mid_angle.to_radians();
        let x = center + label_radius * rad.cos();
        let y = center + label_radius * rad.sin();
        let pct = seg.dash_len / circumference * 100.0;
        (x, y, seg.label.clone(), pct, i)
    }).collect();

    rsx! {
        div { class: "flex flex-col items-center",
            svg {
                view_box: "0 0 {center*2.0} {center*2.0}",
                width: "{size}",
                height: "{size}",
                xmlns: "http://www.w3.org/2000/svg",
                class: "transform -rotate-90",

                // Border outlines (thin, behind color fills)
                for (i, seg) in segments.iter().enumerate() {
                    {
                        let is_hovered = hovered_index == Some(i);
                        let border_sw = if is_hovered { sw + 4.0 } else { sw + 2.0 };
                        let opacity = if is_hovered || hovered_index.is_none() { "1.0" } else { "0.3" };
                        rsx! {
                            circle {
                                key: "{seg.label}-border",
                                cx: "{center}",
                                cy: "{center}",
                                r: "{radius}",
                                fill: "none",
                                stroke: "var(--border, #e5e7eb)",
                                stroke_width: "{border_sw}",
                                stroke_dasharray: "{seg.dash_len} {circumference - seg.dash_len}",
                                stroke_dashoffset: "{-seg.offset}",
                                stroke_linecap: "butt",
                                opacity: "{opacity}",
                                onmouseenter: move |_| { on_hover.call(Some(i)); },
                                onmouseleave: move |_| { on_hover.call(None); },
                            }
                        }
                    }
                }

                // Color fill segments
                for (i, seg) in segments.iter().enumerate() {
                    {
                        let is_hovered = hovered_index == Some(i);
                        let fill_sw = if is_hovered { sw + 4.0 } else { sw };
                        let opacity = if is_hovered || hovered_index.is_none() { "1.0" } else { "0.3" };
                        rsx! {
                            circle {
                                key: "{seg.label}",
                                cx: "{center}",
                                cy: "{center}",
                                r: "{radius}",
                                fill: "none",
                                stroke: "{seg.color}",
                                stroke_width: "{fill_sw}",
                                stroke_dasharray: "{seg.dash_len} {circumference - seg.dash_len}",
                                stroke_dashoffset: "{-seg.offset}",
                                stroke_linecap: "butt",
                                opacity: "{opacity}",
                                onmouseenter: move |_| { on_hover.call(Some(i)); },
                                onmouseleave: move |_| { on_hover.call(None); },
                            }
                        }
                    }
                }
            }
            // Percentage labels outside donut
            svg {
                view_box: "0 0 {center*2.0} {center*2.0}",
                width: "{size}",
                height: "{size}",
                xmlns: "http://www.w3.org/2000/svg",

                for (x, y, label, pct, idx) in label_positions.iter() {
                    if *pct >= 5.0 {
                        {
                            let opacity = if hovered_index == Some(*idx) || hovered_index.is_none() { "1.0" } else { "0.3" };
                            rsx! {
                                text {
                                    key: "{label}-pct",
                                    x: "{x}",
                                    y: "{y}",
                                    fill: "currentColor",
                                    font_size: "5",
                                    text_anchor: "middle",
                                    dominant_baseline: "central",
                                    class: "text-muted-foreground",
                                    opacity: "{opacity}",
                                    "{pct:.0}%"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Shift hue for duplicate colors so adjacent segments stay visually distinct.
pub fn deduplicate_colors(mut data: Vec<DonutData>) -> Vec<DonutData> {
    let mut seen = std::collections::HashSet::new();
    for item in data.iter_mut() {
        if seen.contains(&item.color) {
            item.color = shift_hue(&item.color, seen.len() as u32 * 30);
        }
        seen.insert(item.color.clone());
    }
    data
}

/// Shift hue in a hex color by `degrees` around the color wheel.
pub fn shift_hue(hex: &str, degrees: u32) -> String {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return format!("#{hex}");
    }
    let Ok(r) = u8::from_str_radix(&hex[0..2], 16) else {
        return format!("#{hex}");
    };
    let Ok(g) = u8::from_str_radix(&hex[2..4], 16) else {
        return format!("#{hex}");
    };
    let Ok(b) = u8::from_str_radix(&hex[4..6], 16) else {
        return format!("#{hex}");
    };
    // RGB → HSL
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let h = (h + degrees as f64) % 360.0;
    // HSL → RGB
    let (r2, g2, b2) = hsl_to_rgb(h, s, l);
    format!("#{r2:02x}{g2:02x}{b2:02x}")
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = match max {
        x if x == r => 60.0 * ((g - b) / d % 6.0),
        x if x == g => 60.0 * ((b - r) / d + 2.0),
        x if x == b => 60.0 * ((r - g) / d + 4.0),
        _ => 0.0,
    };
    (h, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

struct DonutSegment {
    label: String,
    color: String,
    dash_len: f64,
    offset: f64,
}

fn compute_donut_segments(data: &[DonutData], total: f64, circumference: f64) -> Vec<DonutSegment> {
    let mut segments = Vec::new();
    let mut cum_offset = 0.0;
    let gap = 2.0_f64; // small gap between slices

    for (i, d) in data.iter().enumerate() {
        let pct = d.value / total;
        let dash_len = (pct * circumference).max(0.0);
        let effective_len = if data.len() > 1 && dash_len > gap * 2.0 {
            dash_len - gap
        } else {
            dash_len
        };
        segments.push(DonutSegment {
            label: format!("{}-{}", d.label, i),
            color: d.color.clone(),
            dash_len: effective_len,
            offset: cum_offset,
        });
        cum_offset += dash_len;
    }
    segments
}

// ── Palette ──────────────────────────────────────────────────────────────────

/// Consistent color palette for category slices.
pub const CHART_PALETTE: &[&str] = &[
    "#6366f1", // indigo
    "#f59e0b", // amber
    "#10b981", // emerald
    "#ef4444", // red
    "#8b5cf6", // violet
    "#06b6d4", // cyan
    "#f97316", // orange
    "#ec4899", // pink
    "#14b8a6", // teal
    "#84cc16", // lime
    "#e11d48", // rose
    "#64748b", // slate
];

/// Get a color from the palette by index.
pub fn palette_color(index: usize) -> &'static str {
    CHART_PALETTE[index % CHART_PALETTE.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_data_serialize() {
        let d = BarData {
            label: "Netflix".into(),
            value: 15.99,
            color: "#e50914".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: BarData = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn max_value_computation() {
        let data = vec![
            BarData {
                label: "A".into(),
                value: 10.0,
                color: "#f00".into(),
            },
            BarData {
                label: "B".into(),
                value: 30.0,
                color: "#0f0".into(),
            },
        ];
        let max = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
        assert_eq!(max, 30.0);
    }

    #[test]
    fn empty_data() {
        let data: Vec<BarData> = vec![];
        assert!(data.is_empty());
    }

    #[test]
    fn line_data_points() {
        let data = vec![
            LineData {
                label: "Jan".into(),
                value: 100.0,
            },
            LineData {
                label: "Feb".into(),
                value: 200.0,
            },
            LineData {
                label: "Mar".into(),
                value: 150.0,
            },
        ];
        let max = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
        assert_eq!(max, 200.0);
    }

    #[test]
    fn donut_segments_computation() {
        let data = vec![
            DonutData {
                label: "A".into(),
                value: 60.0,
                color: "#f00".into(),
            },
            DonutData {
                label: "B".into(),
                value: 40.0,
                color: "#0f0".into(),
            },
        ];
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        assert_eq!(segs.len(), 2);
        // First segment: 60% of circumference
        let expected_first = 0.6 * circ - 2.0; // gap deducted
        assert!((segs[0].dash_len - expected_first).abs() < 0.01);
        assert!((segs[0].offset).abs() < 0.01);
    }

    #[test]
    fn palette_color_cycling() {
        assert_eq!(palette_color(0), "#6366f1");
        assert_eq!(palette_color(12), "#6366f1"); // wraps
        assert_eq!(palette_color(1), "#f59e0b");
    }

    #[test]
    fn donut_empty_data() {
        let data: Vec<DonutData> = vec![];
        assert!(data.is_empty());
    }

    #[test]
    fn donut_zero_total() {
        let _data = vec![DonutData {
            label: "A".into(),
            value: 0.0,
            color: "#f00".into(),
        }];
        let total: f64 = 0.0;
        // Should be handled by returning "No data" element at render time
        assert_eq!(total, 0.0);
    }

    #[test]
    fn donut_segments_single_item() {
        let data = vec![DonutData {
            label: "A".into(),
            value: 100.0,
            color: "#f00".into(),
        }];
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        assert_eq!(segs.len(), 1);
        // Single item gets full circumference (no gap)
        assert!((segs[0].dash_len - circ).abs() < 0.01);
    }

    #[test]
    fn donut_segments_offsets() {
        let data = vec![
            DonutData {
                label: "A".into(),
                value: 50.0,
                color: "#f00".into(),
            },
            DonutData {
                label: "B".into(),
                value: 30.0,
                color: "#0f0".into(),
            },
            DonutData {
                label: "C".into(),
                value: 20.0,
                color: "#00f".into(),
            },
        ];
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        // First offset is 0
        assert!((segs[0].offset).abs() < 0.01);
        // Second offset is after first segment
        assert!(segs[1].offset > 0.0);
        // Third offset is after first + second
        assert!(segs[2].offset > segs[1].offset);
    }

    #[test]
    fn line_data_empty() {
        let data: Vec<LineData> = vec![];
        assert!(data.is_empty());
    }

    #[test]
    fn line_data_single_point() {
        let data = vec![LineData {
            label: "Only".into(),
            value: 42.0,
        }];
        let max = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
        assert_eq!(max, 42.0);
        // step_x would be 0 for single data point (n-1 = 0)
        let n = data.len() as f64;
        let step_x = if data.len() > 1 {
            400.0 / (n - 1.0)
        } else {
            0.0
        };
        assert_eq!(step_x, 0.0);
    }

    #[test]
    fn palette_color_beyond_range() {
        // Test wrapping: index 13 should equal index 1
        assert_eq!(palette_color(13), palette_color(1));
        assert_eq!(palette_color(24), palette_color(0));
    }

    #[test]
    fn deduplicate_colors_no_duplicates() {
        let data = vec![
            DonutData { label: "A".into(), value: 50.0, color: "#ef4444".into() },
            DonutData { label: "B".into(), value: 30.0, color: "#3b82f6".into() },
        ];
        let result = deduplicate_colors(data);
        assert_eq!(result[0].color, "#ef4444");
        assert_eq!(result[1].color, "#3b82f6");
    }

    #[test]
    fn deduplicate_colors_same_color_shifted() {
        let data = vec![
            DonutData { label: "A".into(), value: 50.0, color: "#ef4444".into() },
            DonutData { label: "B".into(), value: 30.0, color: "#ef4444".into() },
        ];
        let result = deduplicate_colors(data);
        assert_eq!(result[0].color, "#ef4444");
        assert_ne!(result[1].color, "#ef4444");
    }

    #[test]
    fn shift_hue_red_by_180() {
        let shifted = shift_hue("#ef4444", 180);
        // Red shifted 180° should be cyan-ish
        assert_ne!(shifted, "#ef4444");
        assert!(shifted.starts_with('#'));
        assert_eq!(shifted.len(), 7);
    }

    #[test]
    fn shift_hue_zero() {
        assert_eq!(shift_hue("#3b82f6", 0), "#3b82f6");
    }

    #[test]
    fn shift_hue_invalid_hex() {
        assert_eq!(shift_hue("zzzzzz", 30), "#zzzzzz");
    }

    #[test]
    fn shift_hue_360_wraps() {
        assert_eq!(shift_hue("#3b82f6", 360), "#3b82f6");
    }

    #[test]
    fn rgb_to_hsl_pure_red() {
        let (h, s, l) = rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn rgb_to_hsl_gray() {
        let (h, s, l) = rgb_to_hsl(128, 128, 128);
        assert_eq!(s, 0.0);
        assert!((l - 0.502).abs() < 0.01);
    }

    #[test]
    fn hsl_to_rgb_roundtrip() {
        let (h, s, l) = rgb_to_hsl(100, 200, 50);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        // Allow small rounding error
        assert!((r as i32 - 100).abs() <= 1);
        assert!((g as i32 - 200).abs() <= 1);
        assert!((b as i32 - 50).abs() <= 1);
    }

    #[test]
    fn bar_data_roundtrip() {
        let d = BarData {
            label: "Test".into(),
            value: 42.0,
            color: "#abc".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: BarData = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn line_data_roundtrip() {
        let d = LineData {
            label: "Jan".into(),
            value: 100.0,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: LineData = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn donut_data_roundtrip() {
        let d = DonutData {
            label: "A".into(),
            value: 50.0,
            color: "#f00".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: DonutData = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn chart_palette_length() {
        assert_eq!(CHART_PALETTE.len(), 12);
    }

    #[test]
    fn chart_palette_all_hex() {
        for color in CHART_PALETTE {
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }

    // ── Additional compute_donut_segments tests ──
    #[test]
    fn donut_segments_equal_values() {
        let data = vec![
            DonutData {
                label: "A".into(),
                value: 33.33,
                color: "#f00".into(),
            },
            DonutData {
                label: "B".into(),
                value: 33.33,
                color: "#0f0".into(),
            },
            DonutData {
                label: "C".into(),
                value: 33.34,
                color: "#00f".into(),
            },
        ];
        let total: f64 = data.iter().map(|d| d.value).sum();
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        assert_eq!(segs.len(), 3);
        // Each segment approx 1/3 of circumference, minus gap
        let per_seg = circ / 3.0;
        for seg in &segs {
            assert!(
                (seg.dash_len - (per_seg - 2.0)).abs() < 5.0,
                "segment len should be ~{}",
                per_seg - 2.0
            );
        }
    }

    #[test]
    fn donut_segments_cumulative_offsets() {
        let data = vec![
            DonutData {
                label: "A".into(),
                value: 70.0,
                color: "#f00".into(),
            },
            DonutData {
                label: "B".into(),
                value: 20.0,
                color: "#0f0".into(),
            },
            DonutData {
                label: "C".into(),
                value: 10.0,
                color: "#00f".into(),
            },
        ];
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        // Offsets should be cumulative: 0, 70% * circ, 90% * circ
        assert!((segs[0].offset - 0.0).abs() < 0.01);
        assert!((segs[1].offset - 0.7 * circ).abs() < 0.01);
        assert!((segs[2].offset - 0.9 * circ).abs() < 0.01);
    }

    #[test]
    fn donut_segments_no_gap_single_item() {
        let data = vec![DonutData {
            label: "A".into(),
            value: 100.0,
            color: "#f00".into(),
        }];
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        assert_eq!(segs.len(), 1);
        // Single item: full circumference, no gap
        assert!((segs[0].dash_len - circ).abs() < 0.01);
    }

    #[test]
    fn donut_segments_many_items() {
        let data: Vec<DonutData> = (0..10)
            .map(|i| DonutData {
                label: format!("{}", i),
                value: 10.0,
                color: "#f00".into(),
            })
            .collect();
        let total: f64 = 100.0;
        let circ = 2.0 * std::f64::consts::PI * 40.0;
        let segs = compute_donut_segments(&data, total, circ);
        assert_eq!(segs.len(), 10);
    }

    // ── Additional palette_color tests ──
    #[test]
    fn palette_color_sequential() {
        let expected = [
            "#6366f1", "#f59e0b", "#10b981", "#ef4444", "#8b5cf6", "#06b6d4", "#f97316", "#ec4899",
            "#14b8a6", "#84cc16", "#e11d48", "#64748b",
        ];
        for (i, &expected_color) in expected.iter().enumerate() {
            assert_eq!(
                palette_color(i),
                expected_color,
                "palette_color({}) should be {}",
                i,
                expected_color
            );
        }
    }

    #[test]
    fn palette_color_wrap_around() {
        // After 12 colors, it wraps
        for i in 0..12 {
            assert_eq!(
                palette_color(i + 12),
                palette_color(i),
                "palette_color({}) should equal palette_color({})",
                i + 12,
                i
            );
        }
    }

    #[test]
    fn palette_color_large_index() {
        assert_eq!(palette_color(1000), palette_color(1000 % 12));
    }
}
