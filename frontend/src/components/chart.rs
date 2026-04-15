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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_data_serialize() {
        let d = BarData { label: "Netflix".into(), value: 15.99, color: "#e50914".into() };
        let json = serde_json::to_string(&d).unwrap();
        let back: BarData = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn max_value_computation() {
        let data = vec![
            BarData { label: "A".into(), value: 10.0, color: "#f00".into() },
            BarData { label: "B".into(), value: 30.0, color: "#0f0".into() },
        ];
        let max = data.iter().map(|d| d.value).fold(0.0_f64, f64::max);
        assert_eq!(max, 30.0);
    }

    #[test]
    fn empty_data() {
        let data: Vec<BarData> = vec![];
        assert!(data.is_empty());
    }
}