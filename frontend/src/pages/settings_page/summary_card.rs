use crate::api::notification_channels;
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::components::timezone_select::TimezoneSelect;
use crate::components::toast::TOAST;
use crate::models::notification::UpdateSummaryNotificationSettingsRequest;
use dioxus::prelude::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::SectionCard;
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
use crate::components::toggle::Toggle;
const WEEKDAYS: &[(i16, &str)] = &[
    (1, "Monday"),
    (2, "Tuesday"),
    (3, "Wednesday"),
    (4, "Thursday"),
    (5, "Friday"),
    (6, "Saturday"),
    (7, "Sunday"),
];

#[component]
pub fn SummaryCard() -> Element {
    let mut enabled = use_signal(|| false);
    let mut frequency = use_signal(|| "monthly".to_string());
    let mut weekday = use_signal(|| 1i16);
    let mut day = use_signal(|| 1i16);
    let mut time = use_signal(|| "09:00".to_string());
    let mut timezone = use_signal(|| "UTC".to_string());
    let mut period = use_signal(|| crate::utils::constants::SUMMARY_PERIOD_CURRENT.to_string());
    let mut next_run = use_signal(|| None::<String>);
    let mut last_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut sending = use_signal(|| false);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        match notification_channels::get_summary().await {
            Ok(summary) => {
                enabled.set(summary.enabled);
                frequency.set(summary.frequency);
                weekday.set(summary.weekday.unwrap_or(1));
                day.set(summary.day_of_month.unwrap_or(1));
                time.set(format!("{:02}:{:02}", summary.hour, summary.minute));
                timezone.set(summary.timezone);
                period.set(summary.period);
                next_run.set(summary.next_run_at.map(|value| value.to_rfc3339()));
                last_error.set(summary.last_error);
            }
            Err(error) => TOAST.write().error(format!(
                "Could not load summary schedule: {error} (API: {})",
                crate::api::client::get_base_url()
            )),
        }
    });

    let send_now = move |_| {
        sending.set(true);
        spawn(async move {
            match notification_channels::send_summary_now().await {
                Ok(()) => TOAST.write().success("Summary sent"),
                Err(error) => TOAST.write().error(format!(
                    "Could not send summary: {error} (API: {})",
                    crate::api::client::get_base_url()
                )),
            }
            sending.set(false);
        });
    };

    let save = move |_| {
        saving.set(true);
        let time_value = time();
        let mut parts = time_value.split(':');
        let hour = parts.next().and_then(|v| v.parse::<i16>().ok()).unwrap_or(9);
        let minute = parts.next().and_then(|v| v.parse::<i16>().ok()).unwrap_or(0);
        let req = UpdateSummaryNotificationSettingsRequest {
            enabled: enabled(),
            frequency: frequency(),
            weekday: Some(weekday()),
            day_of_month: Some(day()),
            hour,
            minute,
            timezone: timezone(),
            period: period(),
        };
        spawn(async move {
            match notification_channels::update_summary(&req).await {
                Ok(summary) => {
                    next_run.set(summary.next_run_at.map(|value| value.to_rfc3339()));
                    last_error.set(summary.last_error);
                    TOAST.write().success("Summary schedule saved");
                }
                Err(error) => TOAST.write().error(format!(
                    "Could not save summary schedule: {error} (API: {})",
                    crate::api::client::get_base_url()
                )),
            }
            saving.set(false);
        });
    };

    rsx! {
        SectionCard {
            title: "Dashboard summary",
            description: "Sent to every active delivery channel",
            content_class: "p-0",
            actions: rsx! {
                Toggle {
                    checked: enabled(),
                    label: "Dashboard summary".to_string(),
                    on_change: move |value| enabled.set(value),
                }
            },
            div { class: "p-6 grid gap-3 sm:grid-cols-2",
                div { class: "space-y-1 text-sm",
                    p { class: "font-medium", "Frequency" }
                    SelectMenu {
                        value: frequency(),
                        options: vec![
                            SelectOption::new("daily", "Daily"),
                            SelectOption::new("weekly", "Weekly"),
                            SelectOption::new("monthly", "Monthly"),
                        ],
                        on_change: move |value| frequency.set(value),
                        searchable: false,
                        class: "mt-1",
                    }
                }
                if frequency() == "weekly" {
                    div { class: "space-y-1 text-sm",
                        p { class: "font-medium", "Weekday" }
                        SelectMenu {
                            value: weekday().to_string(),
                            options: WEEKDAYS.iter().map(|(value, label)| SelectOption::new(value.to_string(), *label)).collect::<Vec<_>>(),
                            on_change: move |value: String| weekday.set(value.parse().unwrap_or(1)),
                            searchable: false,
                            class: "mt-1",
                        }
                    }
                } else if frequency() == "monthly" {
                    div { class: "space-y-1 text-sm",
                        p { class: "font-medium", "Day of month" }
                        SelectMenu {
                            value: day().to_string(),
                            options: {
                                let mut values: Vec<SelectOption> = (1..=crate::utils::constants::MAX_MONTH_DAY)
                                    .map(|value| SelectOption::new(value.to_string(), value.to_string()))
                                    .collect();
                                values.push(SelectOption::new("0", "Last day"));
                                values
                            },
                            on_change: move |value: String| day.set(value.parse().unwrap_or(1)),
                            searchable: false,
                            class: "mt-1",
                        }
                    }
                }
                div { class: "space-y-1 text-sm",
                    p { class: "font-medium", "Report period" }
                    SelectMenu {
                        value: period(),
                        options: vec![
                            SelectOption::new(crate::utils::constants::SUMMARY_PERIOD_CURRENT, "Current month"),
                            SelectOption::new(crate::utils::constants::SUMMARY_PERIOD_NEXT, "Next month"),
                        ],
                        on_change: move |value| period.set(value),
                        searchable: false,
                        class: "mt-1",
                    }
                }
                Field { label: "Time", html_for: "summary-time", class: "text-sm",
                    Input {
                        id: "summary-time",
                        r#type: "time",
                        size: InputSize::Sm,
                        value: time(),
                        on_input: move |value| time.set(value),
                    }
                }
                div { class: "space-y-1 text-sm",
                    p { class: "font-medium", "Timezone" }
                    TimezoneSelect { value: timezone }
                }
            }
            div { class: "px-6 pb-6 flex flex-wrap items-center justify-between gap-3",
                div {
                    if let Some(next) = next_run() {
                        p { class: "text-xs text-muted-foreground", "Next run: {next}" }
                    } else {
                        p { class: "text-xs text-muted-foreground", "No run scheduled" }
                    }
                    if let Some(error) = last_error() {
                        p { class: "text-xs text-destructive mt-1", "Last attempt failed: {error}" }
                    }
                }
                div { class: "flex flex-wrap items-center gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: sending(),
                        onclick: send_now,
                        if sending() { "Sending..." } else { "Send now" }
                    }
                    Button {
                        disabled: saving(),
                        onclick: save,
                        if saving() { "Saving..." } else { "Save schedule" }
                    }
                }
            }
        }
    }
}
