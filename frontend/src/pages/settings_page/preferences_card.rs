use crate::api::settings;
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::components::toast::TOAST;
use crate::models::auth::UpdateSettingsRequest;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::utils::constants::CURRENCIES;
use dioxus::prelude::*;

/// General preferences: currency, first day of week, date format and updates.
use crate::components::card::SectionCard;
use crate::components::checkbox::Checkbox;
use crate::components::field::Field;
#[component]
pub fn PreferencesCard() -> Element {
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut saving_currency = use_signal(|| false);

    // Sync currency if it is empty (never overwrite the user's in-progress edit).
    use_effect(move || {
        if let Some(u) = &AUTH().user
            && currency().is_empty()
        {
            currency.set(u.currency.clone());
        }
    });

    let mut handle_save_currency = move |new_currency: String| {
        currency.set(new_currency.clone());
        saving_currency.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                email: None,
                avatar: None,
                name: None,
                currency: Some(new_currency),
            })
            .await
            {
                Ok(resp) => {
                    AUTH.write().user = Some(resp.user);
                    AUTH.write().persist_to_storage();
                    TOAST.write().success("Currency updated");
                    saving_currency.set(false);
                }
                Err(e) => {
                    log::error!("Failed to save currency: {}", e);
                    TOAST.write().error("Failed to update currency");
                    saving_currency.set(false);
                }
            }
        });
    };

    let cur = currency();

    rsx! {
        SectionCard {
            title: "Defaults",
            description: "Applied to new entries and reports",
            content_class: "p-0",
            div { class: "p-6 space-y-4",
                Field { label: "Default Currency", html_for: "settings-currency",
                    SelectMenu {
                        id: "settings-currency",
                        value: cur,
                        options: CURRENCIES.iter().map(|(value, label)| SelectOption::new(*value, *label)).collect::<Vec<_>>(),
                        on_change: move |value: String| handle_save_currency(value),
                        searchable: true,
                        disabled: saving_currency(),
                    }
                }
                Field { label: "First Day of Week", html_for: "settings-first-day",
                    SelectMenu {
                        id: "settings-first-day",
                        value: PREFERENCES().first_day_of_week().to_string(),
                        options: vec![
                            SelectOption::new("monday", "Monday"),
                            SelectOption::new("sunday", "Sunday"),
                        ],
                        on_change: move |day: String| {
                            let previous = PREFERENCES().first_day_of_week().to_string();
                            PREFERENCES.write().set_first_day_of_week(day.clone());
                            spawn(async move {
                                match crate::api::preferences::update(&crate::models::preferences::UpdatePreferencesRequest {
                                    theme: None,
                                    subscriptions_view: None,
                                    first_day_of_week: Some(day),
                                    date_format: None,
                                }).await {
                                    Ok(_) => TOAST.write().success("First day of week updated"),
                                    Err(e) => {
                                        log::error!("Failed to update first_day_of_week: {:?}", e);
                                        PREFERENCES.write().set_first_day_of_week(previous);
                                        TOAST.write().error("Failed to update preference");
                                    }
                                }
                            });
                        },
                        searchable: false,
                    }
                }
                Field { label: "Date format", html_for: "settings-date-format",
                    SelectMenu {
                        id: "settings-date-format",
                        value: PREFERENCES().date_format(),
                        options: vec![
                            SelectOption::new("DD/MM/YYYY", "DD/MM/YYYY"),
                            SelectOption::new("MM/DD/YYYY", "MM/DD/YYYY"),
                            SelectOption::new("YYYY/MM/DD", "YYYY/MM/DD"),
                        ],
                        on_change: move |format: String| {
                            let previous = PREFERENCES().date_format();
                            PREFERENCES.write().set_date_format(format.clone());
                            spawn(async move {
                                match crate::api::preferences::update(&crate::models::preferences::UpdatePreferencesRequest {
                                    theme: None,
                                    subscriptions_view: None,
                                    first_day_of_week: None,
                                    date_format: Some(format),
                                }).await {
                                    Ok(_) => TOAST.write().success("Date format updated"),
                                    Err(error) => {
                                        log::error!("Failed to update date format: {error}");
                                        PREFERENCES.write().set_date_format(previous);
                                        TOAST.write().error("Failed to update date format");
                                    }
                                }
                            });
                        },
                        searchable: false,
                    }
                }
                div { class: "flex items-start gap-3 rounded-lg border border-border/40 p-3",
                    Checkbox {
                        id: "settings-show-updates",
                        checked: PREFERENCES().show_updates(),
                        on_change: move |checked| PREFERENCES.write().set_show_updates(checked),
                        class: "mt-0.5",
                    }
                    div { class: "space-y-0.5",
                        label { r#for: "settings-show-updates", class: "text-sm font-medium", "Show update notices" }
                        p { class: "text-xs text-muted-foreground", "Check for new Recurrency releases and show a dismissible banner." }
                    }
                }
            }
        }
    }
}
