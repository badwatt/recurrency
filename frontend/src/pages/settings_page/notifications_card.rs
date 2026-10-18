use super::{NOTIFICATION_CHANNELS, is_email_disabled};
use crate::api::app;
use crate::api::notification_preferences;
use crate::components::toast::TOAST;
use crate::models::notification::{
    UpdateNotificationPreferenceRequest, UpdateNotificationPreferencesRequest,
};
use dioxus::prelude::*;

/// Notifications card: per-channel toggles, disabling email when SMTP is absent.
#[component]
pub fn NotificationsCard() -> Element {
    let mut notification_prefs = use_signal(|| {
        let mut map = std::collections::HashMap::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            map.insert(key.to_string(), false);
        }
        map
    });
    let mut prefs_loading = use_signal(|| false);
    let mut smtp_configured = use_signal(|| true); // Default true to avoid flash

    // Load notification preferences
    use_future(move || async move {
        match notification_preferences::get().await {
            Ok(resp) => {
                let mut map = std::collections::HashMap::new();
                for p in resp.preferences {
                    map.insert(p.channel, p.enabled);
                }
                notification_prefs.set(map);
            }
            Err(e) => log::error!("Failed to load notification prefs: {}", e),
        }
    });

    // Load app info to check SMTP configuration
    use_future(move || async move {
        match app::get_app_info().await {
            Ok(info) => smtp_configured.set(info.smtp_configured),
            Err(e) => log::error!("Failed to load app info: {}", e),
        }
    });

    let mut handle_notification_toggle = move |channel: String, enabled: bool| {
        let mut prefs = notification_prefs();
        prefs.insert(channel.clone(), enabled);
        notification_prefs.set(prefs);
        prefs_loading.set(true);
        spawn(async move {
            let req = UpdateNotificationPreferencesRequest {
                preferences: vec![UpdateNotificationPreferenceRequest {
                    channel: channel.clone(),
                    enabled,
                }],
            };
            match notification_preferences::update(&req).await {
                Ok(_) => {
                    TOAST.write().success("Notification preferences updated");
                }
                Err(e) => {
                    log::error!("Failed to update notification pref: {}", e);
                    let mut prefs = notification_prefs();
                    prefs.insert(channel, !enabled);
                    notification_prefs.set(prefs);
                    TOAST.write().error("Failed to update notifications");
                }
            }
            prefs_loading.set(false);
        });
    };

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Notifications" }
                p { class: "text-sm text-muted-foreground", "Configure how you receive notifications" }
            }
            div { class: "p-6 space-y-4",
                for (idx, (key, label, desc)) in NOTIFICATION_CHANNELS.iter().enumerate() {
                    {let checked = notification_prefs().get(*key).copied().unwrap_or(false);
                     let key_str = key.to_string();
                     let is_email_disabled = is_email_disabled(key, smtp_configured());
                     rsx! {
                        div { key: "{key}",
                            div { class: "flex items-center justify-between gap-4",
                                div { class: "min-w-0",
                                    p { class: "font-medium truncate", {label.to_string()} }
                                    p { class: "text-sm text-muted-foreground truncate", {desc.to_string()} }
                                    if is_email_disabled {
                                        div { class: "flex items-center gap-1.5 text-amber-600 mt-1",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "14",
                                                height: "14",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                polygon { points: "7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2" }
                                                line { x1: "12", y1: "8", x2: "12", y2: "12" }
                                                line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
                                            }
                                            p { class: "text-xs font-medium", "SMTP not configured — email notifications unavailable" }
                                        }
                                    }
                                }
                                button {
                                    class: if checked {
                                        "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors focus:outline-none shrink-0"
                                    } else {
                                        "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors focus:outline-none shrink-0"
                                    },
                                    onclick: {
                                        let key_str = key_str.clone();
                                        let new_val = !checked;
                                        move |_| handle_notification_toggle(key_str.clone(), new_val)
                                    },
                                    disabled: prefs_loading() || is_email_disabled,
                                    span {
                                        class: if checked {
                                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                        } else {
                                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                        },
                                    }
                                }
                            }
                            if idx < NOTIFICATION_CHANNELS.len() - 1 {
                                hr { class: "mt-4 border-border" }
                            }
                        }
                    }}
                }
            }
        }
    }
}
