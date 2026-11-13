use crate::api::notification_channels;
use crate::components::toast::TOAST;
use crate::models::notification::UpdateSummaryNotificationSettingsRequest;
use dioxus::prelude::*;

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
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                div { class: "flex items-center justify-between gap-4",
                    div {
                        h2 { class: "text-lg font-semibold", "Dashboard summary" }
                        p { class: "text-sm text-muted-foreground", "Sent to every active delivery channel" }
                    }
                    button {
                        r#type: "button",
                        class: if enabled() { "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors" } else { "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors" },
                        onclick: move |_| enabled.toggle(),
                        span {
                            class: if enabled() { "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6" } else { "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1" }
                        }
                    }
                }
            }
            div { class: "p-6 grid gap-3 sm:grid-cols-2",
                label { class: "space-y-1 text-sm", "Frequency"
                    select {
                        value: frequency(),
                        onchange: move |e| frequency.set(e.value()),
                        class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                        option { value: "daily", "Daily" }
                        option { value: "weekly", "Weekly" }
                        option { value: "monthly", "Monthly" }
                    }
                }
                if frequency() == "weekly" {
                    label { class: "space-y-1 text-sm", "Weekday"
                        select {
                            value: weekday().to_string(),
                            onchange: move |e| weekday.set(e.value().parse().unwrap_or(1)),
                            class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                            for (value, label) in WEEKDAYS {
                                option { value: value.to_string(), {label.to_string()} }
                            }
                        }
                    }
                } else if frequency() == "monthly" {
                    label { class: "space-y-1 text-sm", "Day of month"
                        select {
                            value: day().to_string(),
                            onchange: move |e| day.set(e.value().parse().unwrap_or(1)),
                            class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                            for value in 1..=crate::utils::constants::MAX_MONTH_DAY {
                                option { value: value.to_string(), "{value}" }
                            }
                            option { value: "0", "Last day" }
                        }
                    }
                }
                label { class: "space-y-1 text-sm", "Report period"
                    select {
                        value: period(),
                        onchange: move |e| period.set(e.value()),
                        class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                        option { value: crate::utils::constants::SUMMARY_PERIOD_CURRENT, "Current month" }
                        option { value: crate::utils::constants::SUMMARY_PERIOD_NEXT, "Next month" }
                    }
                }
                label { class: "space-y-1 text-sm", "Time"
                    input {
                        r#type: "time",
                        value: time(),
                        oninput: move |e| time.set(e.value()),
                        class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                    }
                }
                label { class: "space-y-1 text-sm", "Timezone"
                    input {
                        value: timezone(),
                        oninput: move |e| timezone.set(e.value()),
                        placeholder: "Europe/Madrid",
                        class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                    }
                }
            }
            div { class: "px-6 pb-6 flex flex-wrap items-center justify-between gap-3",
                if let Some(next) = next_run() {
                    p { class: "text-xs text-muted-foreground", "Next run: {next}" }
                } else {
                    p { class: "text-xs text-muted-foreground", "No run scheduled" }
                }
                div { class: "flex flex-wrap items-center gap-2",
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md border border-border px-4 py-2 text-sm font-medium hover:bg-accent disabled:opacity-50",
                        disabled: sending(),
                        onclick: send_now,
                        if sending() { "Sending..." } else { "Send now" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: saving(),
                        onclick: save,
                        if saving() { "Saving..." } else { "Save schedule" }
                    }
                }
            }
        }
    }
}
