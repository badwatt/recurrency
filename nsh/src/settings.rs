//! Settings: account, appearance, server and about.

use nsh_base::{button, theme};
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::AiSettings;

use crate::auth::{self, AuthSession, AuthUser, Session};
use crate::ui;

#[derive(Debug, Deserialize)]
pub struct SettingsResponse {
    pub user: AuthUser,
}

#[derive(Debug, Serialize)]
pub struct UpdateSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Account,
    Appearance,
    General,
    Notifications,
    Ai,
    Money,
    Data,
    Server,
    Admin,
    About,
}

impl Section {
    const ALL: [Section; 10] = [
        Section::Account,
        Section::Appearance,
        Section::General,
        Section::Notifications,
        Section::Ai,
        Section::Money,
        Section::Data,
        Section::Server,
        Section::Admin,
        Section::About,
    ];

    fn label(self) -> &'static str {
        match self {
            Section::Account => "Account",
            Section::Appearance => "Appearance",
            Section::General => "General",
            Section::Notifications => "Notifications",
            Section::Ai => "AI assistant",
            Section::Money => "Accounts & categories",
            Section::Data => "Data & backup",
            Section::Server => "Server",
            Section::Admin => "Administration",
            Section::About => "About",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Account => "Profile, session and account",
            Section::Appearance => "Color theme and palette",
            Section::General => "Currency and defaults",
            Section::Notifications => "Delivery channels",
            Section::Ai => "Provider and model",
            Section::Money => "Accounts and categories",
            Section::Data => "Backups and restore",
            Section::Server => "Connection and device pairing",
            Section::Admin => "Users and registration",
            Section::About => "Version and release notes",
        }
    }
}

pub fn settings(auth: AuthSession, session: Signal<Session>) -> AnyElement {
    let section = signal(Section::Account);
    let name = signal(auth.user.name.clone().unwrap_or_default());
    let currency = signal(auth.user.currency.clone());
    let error = signal(String::new());
    let saving = signal(false);
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let email = auth.user.email.clone();
    let verified = auth.user.verified;
    let is_admin = auth.user.is_admin;
    let backup_status = signal(String::new());
    let save_server = server.clone();
    let save_token = token.clone();
    let data_server = signal(server.clone());
    let data_token = signal(token.clone());
    let info_server = server.clone();

    let save = {
        let token = save_token;
        let server = save_server;
        move || {
            if saving.get() {
                return;
            }
            saving.set(true);
            error.set(String::new());
            let request = UpdateSettingsRequest {
                name: Some(name.get().trim().to_string()),
                currency: Some(currency.get().trim().to_uppercase()),
            };
            let server_value = server.clone();
            let token_value = token.clone();
            let auth = auth.clone();
            spawn(async move {
                match crate::api::update_settings(server_value, token_value, &request).await {
                    Ok(response) => {
                        let next = AuthSession {
                            token: auth.token,
                            user: response.user,
                            requires_verification: false,
                        };
                        auth::store_session(&next);
                        session.set(Session::SignedIn(next));
                    }
                    Err(message) => error.set(message),
                }
                saving.set(false);
            });
        }
    };

    let save_general = save.clone();
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(px(28.0))
        .child(
            view()
                .w(px(220.0))
                .flex_col()
                .gap(px(4.0))
                .child(
                    text("SETTINGS")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .mb(px(8.0)),
                )
                .children(Section::ALL.iter().filter(|item| **item != Section::Admin || is_admin).map(|item| {
                    let item = *item;
                    let active = move || section.get() == item;
                    button()
                        .w_full()
                        .rounded(px(8.0))
                        .px(px(10.0))
                        .py(px(8.0))
                        .bg(move || if active() { theme().accent } else { theme().background })
                        .on_press(move || section.set(item))
                        .child(
                            view()
                                .flex_col()
                                .gap(px(1.0))
                                .child(
                                    text(item.label())
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || if active() {
                                            theme().foreground
                                        } else {
                                            theme().muted_foreground
                                        }),
                                )
                                .child(
                                    text(item.description())
                                        .text_size(px(10.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                })),
        )
        .child(
            view()
                .flex_1()
                .min_w(px(320.0))
                .flex_col()
                .gap(px(20.0))
                .child(ui::page_header("Settings", "Preferences for this device"))
                .child(
                    when(move || section.get() == Section::Account, move || {
                        account_card(
                            email.clone(),
                            verified,
                            is_admin,
                            name,
                            currency,
                            error,
                            saving,
                            save.clone(),
                            session,
                        )
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Appearance, || appearance_card())
                        .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::General, move || {
                        general_card(currency, save_general.clone())
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Notifications, move || {
                        notifications_card(data_server.get(), data_token.get())
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Ai, move || {
                        ai_card(data_server.get(), data_token.get())
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Money, move || {
                        money_card(data_server.get(), data_token.get())
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Admin, move || {
                        admin_card(data_server.get(), data_token.get())
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Data, move || {
                        data_card(data_server.get(), data_token.get(), backup_status)
                    })
                        .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Server, move || server_card(info_server.clone()))
                        .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::About, || about_card())
                        .otherwise(|| view().h(px(0.0))),
                ),
        )
        .into_element()
}

fn account_card(
    email: String,
    verified: bool,
    is_admin: bool,
    name: Signal<String>,
    currency: Signal<String>,
    error: Signal<String>,
    saving: Signal<bool>,
    save: impl Fn() + 'static,
    session: Signal<Session>,
) -> impl IntoElement {
    view()
        .w_full()
        .max_w(px(680.0))
        .flex_col()
        .gap(px(16.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(16.0))
                .child(
                    text("Account")
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            text("Email")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text(email)
                                .text_size(px(14.0))
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::field("Name", name, "Your name"))
                        .child(ui::field("Currency", currency, "EUR")),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(8.0))
                        .child(
                            text(if verified { "Email verified" } else { "Email not verified" })
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .when(is_admin, |row| {
                            row.child(
                                text("Administrator")
                                    .text_size(px(11.0))
                                    .text_color(move || theme().primary),
                            )
                        }),
                )
                .child(
                    when(move || !error.get().is_empty(), move || {
                        text(move || error.get())
                            .text_size(px(12.0))
                            .text_color(move || theme().destructive)
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(8.0))
                        .child(
                            button()
                                .h(px(36.0))
                                .px(px(14.0))
                                .rounded(px(7.0))
                                .bg(move || theme().primary)
                                .disabled(move || saving.get())
                                .disabled_style(|state| state.opacity(0.5))
                                .on_press(move || save())
                                .child(
                                    text(move || if saving.get() { "Saving..." } else { "Save changes" })
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().primary_foreground),
                                ),
                        )
                        .child(ui::action_button("Log out", true, move || {
                            auth::clear_session();
                            session.set(Session::SignedOut);
                        })),
                ),
        )
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(6.0))
                .child(
                    text("Session")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Signing out removes the token from this device. Your server data stays untouched.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
}

fn appearance_card() -> impl IntoElement {
    ui::surface()
        .p(px(24.0))
        .gap(px(8.0))
        .child(
            text("Appearance")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text("Mocha dark palette")
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            text("Native Shell uses the app theme tokens. The full 18-palette catalogue is being ported.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn general_card(currency: Signal<String>, save: impl Fn() + 'static) -> impl IntoElement {
    ui::surface()
        .w(px(680.0))
        .p(px(24.0))
        .gap(px(14.0))
        .child(
            text("General")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(ui::field("Display currency", currency, "EUR"))
        .child(
            text("Amounts are formatted in this currency. Exchange rates stay on the server.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            button()
                .h(px(36.0))
                .px(px(14.0))
                .rounded(px(7.0))
                .bg(move || theme().primary)
                .on_press(move || save())
                .child(
                    text("Save changes")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
}

fn data_card(
    server: String,
    token: String,
    status: Signal<String>,
) -> impl IntoElement {
    let export = {
        let server = server.clone();
        let token = token.clone();
        move || {
            status.set("Preparing backup…".to_string());
            let server_value = server.clone();
            let token_value = token.clone();
            spawn(async move {
                match crate::api::export_backup(server_value, token_value).await {
                    Ok(json) => match pick_save_path(
                        SaveDialog::new()
                            .file_name("recurrency-backup.json")
                            .filter(FileFilter::new("JSON", ["json"])),
                    )
                    .await
                    {
                        Ok(Some(path)) => match std::fs::write(&path, json) {
                            Ok(()) => status.set(format!("Backup saved to {}", path.display())),
                            Err(error) => status.set(format!("Could not write the backup: {error}")),
                        },
                        Ok(None) => status.set(String::new()),
                        Err(error) => status.set(error),
                    },
                    Err(error) => status.set(error),
                }
            });
        }
    };
    ui::surface()
        .p(px(24.0))
        .gap(px(10.0))
        .child(
            text("Data & backup")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text("Download a validated JSON backup created by the server. Restore uses the admin endpoints.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(ui::action_button("Export backup", false, export))
        .child(
            text(move || status.get())
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn server_card(server: String) -> impl IntoElement {
    ui::surface()
        .p(px(24.0))
        .gap(px(8.0))
        .child(
            text("Server")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(server)
                .text_size(px(13.0))
                .font_family(crate::theme::MONO)
                .text_color(move || theme().foreground),
        )
        .child(
            text("The native app connects to this server. Change it from the sign-in screen.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn about_card() -> impl IntoElement {
    ui::surface()
        .p(px(24.0))
        .gap(px(6.0))
        .child(
            text("About")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(format!("Recurrency v{}", env!("CARGO_PKG_VERSION")))
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            text("Native Shell migration · Rust · Axum · PostgreSQL")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
}

#[derive(Serialize)]
struct UpdateNotificationPreferencesRequest {
    preferences: Vec<UpdateNotificationPreference>,
}

#[derive(Serialize)]
struct UpdateNotificationPreference {
    channel: String,
    enabled: bool,
    reminder_days_before: i32,
}

#[derive(Serialize)]
struct UpdateAiSettingsRequest {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
}

fn notifications_card(server: String, token: String) -> impl IntoElement {
    let data = resource(
        || Some(()),
        {
            let server = server.clone();
            let token = token.clone();
            move |(), _| {
                let server = server.clone();
                let token = token.clone();
                async move { crate::api::load_notification_preferences(server, token).await }
            }
        },
    );
    let error = signal(String::new());
    let channel_currency = server.clone();
    let channel_token = token.clone();
    view()
        .w(px(680.0))
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(10.0))
                .child(
                    text("Notifications")
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Choose which channels the server may use for reminders and summaries.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || {
                                data.get()
                                    .unwrap_or_default()
                                    .preferences
                            },
                            |preference| preference.channel.clone(),
                            move |preference, _| {
                                let preference = preference.get();
                                let channel = preference.channel.clone();
                                let toggle_channel = channel.clone();
                                let enabled = preference.enabled;
                                let days = preference.reminder_days_before;
                                let server = channel_currency.clone();
                                let token = channel_token.clone();
                                let data_toggle = data;
                                let toggle = move || {
                                    let server = server.clone();
                                    let token = token.clone();
                                    let request = UpdateNotificationPreferencesRequest {
                                        preferences: vec![UpdateNotificationPreference {
                                            channel: toggle_channel.clone(),
                                            enabled: !enabled,
                                            reminder_days_before: days,
                                        }],
                                    };
                                    spawn(async move {
                                        let _ = crate::api::update_notification_preferences(
                                            server, token, &request,
                                        )
                                        .await;
                                        data_toggle.refetch();
                                    });
                                };
                                view()
                                    .w_full()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .py(px(8.0))
                                    .border_b(px(1.0))
                                    .border_color(move || theme().border)
                                    .child(
                                        view()
                                            .flex_col()
                                            .child(
                                                text(channel_label(&channel))
                                                    .text_size(px(13.0))
                                                    .text_color(move || theme().foreground),
                                            )
                                            .child(
                                                text(format!("{days} days before"))
                                                    .text_size(px(10.0))
                                                    .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(ui::action_button(
                                        if enabled { "Disable" } else { "Enable" },
                                        false,
                                        toggle,
                                    ))
                            },
                        )),
                )
                .child(
                    when(move || !error.get().is_empty(), move || {
                        text(move || error.get())
                            .text_size(px(12.0))
                            .text_color(move || theme().destructive)
                    })
                    .otherwise(|| view().h(px(0.0))),
                ),
        )
}

fn channel_label(channel: &str) -> String {
    match channel {
        "email" => "Email".to_string(),
        "push" => "Push".to_string(),
        "in_app" | "in-app" => "In-app".to_string(),
        other => other.to_string(),
    }
}

fn ai_card(server: String, token: String) -> impl IntoElement {
    let data = resource(
        || Some(()),
        {
            let server = server.clone();
            let token = token.clone();
            move |(), _| {
                let server = server.clone();
                let token = token.clone();
                async move { crate::api::load_ai_settings(server, token).await }
            }
        },
    );
    let provider = signal(String::new());
    let base_url = signal(String::new());
    let model = signal(String::new());
    let api_key = signal(String::new());
    let status = signal(String::new());
    let save = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            let enabled = data
                .get()
                .map_or(false, |settings: AiSettings| settings.enabled);
            let request = UpdateAiSettingsRequest {
                enabled: !enabled,
                provider: (!provider.get().trim().is_empty()).then(|| provider.get()),
                base_url: (!base_url.get().trim().is_empty()).then(|| base_url.get()),
                model: (!model.get().trim().is_empty()).then(|| model.get()),
                api_key: (!api_key.get().trim().is_empty()).then(|| api_key.get()),
            };
            spawn(async move {
                match crate::api::update_ai_settings(server, token, &request).await {
                    Ok(_) => data.refetch(),
                    Err(message) => status.set(message),
                }
            });
        }
    };
    let configure = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            let enabled = data
                .get()
                .map_or(false, |settings: AiSettings| settings.enabled);
            let request = UpdateAiSettingsRequest {
                enabled,
                provider: (!provider.get().trim().is_empty()).then(|| provider.get()),
                base_url: (!base_url.get().trim().is_empty()).then(|| base_url.get()),
                model: (!model.get().trim().is_empty()).then(|| model.get()),
                api_key: (!api_key.get().trim().is_empty()).then(|| api_key.get()),
            };
            spawn(async move {
                match crate::api::update_ai_settings(server, token, &request).await {
                    Ok(_) => {
                        status.set("AI settings saved.".to_string());
                        data.refetch();
                    }
                    Err(message) => status.set(message),
                }
            });
        }
    };
    view()
        .w(px(680.0))
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(12.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text("AI assistant")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || {
                                data.get()
                                    .map_or("Not configured".to_string(), |settings| {
                                        if settings.configured {
                                            "Configured".to_string()
                                        } else {
                                            "Not configured".to_string()
                                        }
                                    })
                            })
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    text("Bring your own key. The key is encrypted on the server and never shown again.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::field("Provider", provider, "openai"))
                        .child(ui::field("Base URL", base_url, "https://api.openai.com/v1")),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::field("Model", model, "gpt-4o-mini"))
                        .child(ui::field("API key", api_key, "sk-…")),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(8.0))
                        .child(ui::action_button("Save", false, configure))
                        .child(
                            button()
                                .h(px(26.0))
                                .px(px(8.0))
                                .rounded(px(5.0))
                                .bg(move || theme().card)
                                .on_press(move || save())
                                .child(
                                    text(move || {
                                        data.get().map_or("Enable", |settings| {
                                            if settings.enabled { "Disable" } else { "Enable" }
                                        })
                                    })
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    text(move || status.get())
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
}

fn money_card(server: String, token: String) -> impl IntoElement {
    let data = resource(
        || Some(()),
        {
            let server = server.clone();
            let token = token.clone();
            move |(), _| {
                let server = server.clone();
                let token = token.clone();
                async move { crate::api::load_money_counts(server, token).await }
            }
        },
    );
    view()
        .w(px(680.0))
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(16.0))
                .child(
                    text("Accounts")
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || {
                                data.get()
                                    .map_or_else(Vec::new, |(accounts, _)| accounts)
                            },
                            |item| item.account.id,
                            move |item, _| {
                                let item = item.get();
                                view()
                                    .w_full()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .py(px(7.0))
                                    .border_b(px(1.0))
                                    .border_color(move || theme().border)
                                    .child(
                                        text(item.account.name)
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                    )
                                    .child(
                                        text(format!(
                                            "{} subscriptions · {} transactions",
                                            item.subscription_count, item.transaction_count
                                        ))
                                        .text_size(px(10.0))
                                        .text_color(move || theme().muted_foreground),
                                    )
                            },
                        )),
                )
                .child(
                    text("Categories")
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || {
                                data.get()
                                    .map_or_else(Vec::new, |(_, categories)| categories)
                            },
                            |item| item.category.id,
                            move |item, _| {
                                let item = item.get();
                                view()
                                    .w_full()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .py(px(7.0))
                                    .border_b(px(1.0))
                                    .border_color(move || theme().border)
                                    .child(
                                        text(item.category.name)
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                    )
                                    .child(
                                        text(format!(
                                            "{} subscriptions · {} transactions",
                                            item.subscription_count, item.transaction_count
                                        ))
                                        .text_size(px(10.0))
                                        .text_color(move || theme().muted_foreground),
                                    )
                            },
                        )),
                ),
        )
}

fn admin_card(server: String, token: String) -> impl IntoElement {
    let data = resource(
        || Some(()),
        {
            let server = server.clone();
            let token = token.clone();
            move |(), _| {
                let server = server.clone();
                let token = token.clone();
                async move { crate::api::load_admin_users(server, token).await }
            }
        },
    );
    ui::surface()
        .w(px(680.0))
        .p(px(24.0))
        .gap(px(10.0))
        .child(
            text("Administration")
                .text_size(px(18.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text("Users registered on this server.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_col()
                .gap(px(6.0))
                .child(each(
                    move || data.get().unwrap_or_default(),
                    |user| user.id,
                    move |user, _| {
                        let user = user.get();
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .py(px(7.0))
                            .border_b(px(1.0))
                            .border_color(move || theme().border)
                            .child(
                                view()
                                    .flex_col()
                                    .child(
                                        text(user.name.unwrap_or_else(|| user.email.clone()))
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                    )
                                    .child(
                                        text(user.email)
                                            .text_size(px(10.0))
                                            .text_color(move || theme().muted_foreground),
                                    ),
                            )
                            .child(
                                text(if user.is_admin { "Admin" } else { "User" })
                                    .text_size(px(10.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                    },
                )),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_sections_have_distinct_labels() {
        let mut labels: Vec<&str> = Section::ALL.iter().map(|section| section.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), Section::ALL.len());
    }

    #[test]
    fn update_request_skips_empty_fields() {
        let request = UpdateSettingsRequest {
            name: None,
            currency: None,
        };
        let value = serde_json::to_value(request).expect("settings serializes");
        assert_eq!(value.as_object().expect("object").len(), 0);
    }
}
