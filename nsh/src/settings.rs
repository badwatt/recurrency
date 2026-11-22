//! Settings: account, appearance, server and about.

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::AiSettings;

use crate::auth::{self, AuthSession, AuthUser, Session};
use uuid::Uuid;
use crate::components::toggle::toggle as shared_toggle;
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
    #[allow(dead_code)]
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

    fn icon(self) -> ImageAsset {
        match self {
            Section::Account => asset!("icons/settings.svg"),
            Section::Appearance => asset!("icons/palette.svg"),
            Section::General => asset!("icons/sun.svg"),
            Section::Notifications => asset!("icons/bell.svg"),
            Section::Ai => asset!("icons/sparkles.svg"),
            Section::Money => asset!("icons/tag.svg"),
            Section::Data => asset!("icons/database.svg"),
            Section::Server => asset!("icons/monitor.svg"),
            Section::Admin => asset!("icons/settings.svg"),
            Section::About => asset!("icons/info.svg"),
        }
    }

    fn title(self) -> &'static str {
        self.label()
    }

    fn subtitle(self) -> &'static str {
        match self {
            Section::Account => "Your profile, session and account",
            Section::Appearance => "Color themes and palette",
            Section::General => "Currency, exchange rates, week start and dates",
            Section::Notifications => "Choose where your summary lands and when it leaves",
            Section::Ai => "Model, voice input and privacy",
            Section::Money => "Accounts and categories that organize your money",
            Section::Data => "Keep a copy of your data and sync devices",
            Section::Server => "Server connection and device pairing",
            Section::Admin => "Users, registration and full backups",
            Section::About => "Version, release notes and Android app",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Account => "Profile, session and account removal",
            Section::Appearance => "Color theme and palette",
            Section::General => "Currency, exchange rates, week start and dates",
            Section::Notifications => "Delivery channels and summaries",
            Section::Ai => "Model, voice input and privacy",
            Section::Money => "Accounts and categories",
            Section::Data => "Backups, restore and folder sync",
            Section::Server => "Server connection and device pairing",
            Section::Admin => "Users and registration",
            Section::About => "Version, release notes and Android app",
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
    let delete_open = signal(false);
    let delete_password = signal(String::new());
    let delete_error = signal(String::new());
    let save_server = server.clone();
    let save_token = token.clone();
    let data_server = signal(server.clone());
    let data_token = signal(token.clone());
    let info_server = server.clone();
    let appearance_server = server.clone();
    let appearance_token = token.clone();

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
                        saving.set(false);
                        session.set(Session::SignedIn(next));
                    }
                    Err(message) => {
                        error.set(message);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_general = save.clone();
    view()
        .w_full()
        .pt(px(10.0))
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(px(36.0))
        .child(
            view()
                .w(px(200.0))
                .flex_col()
                .gap(px(4.0))
                .child(
                    text("SETTINGS")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .mb(px(8.0)),
                )
                .children([
                    ("PERSONAL", &[Section::Account, Section::Appearance][..]),
                    ("CONFIGURATION", &[Section::General, Section::Ai, Section::Money, Section::Notifications][..]),
                    ("SYSTEM", &[Section::Data, Section::Server, Section::Admin, Section::About][..]),
                ].into_iter().map(|(group, items)| {
                    view()
                        .flex_col()
                        .gap(px(2.0))
                        .mb(px(8.0))
                        .child(
                            text(group)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground)
                                .px(px(10.0))
                                .mb(px(2.0)),
                        )
                        .children(items.iter().filter(|item| **item != Section::Admin || is_admin).map(|item| {
                            let item = *item;
                            let active = move || section.get() == item;
                            button()
                                .items_center()
                                .justify_start()
                                .w_full()
                                .rounded(px(8.0))
                                .px(px(10.0))
                                .py(px(10.0))
                                .bg(move || if active() { theme().accent } else { theme().background })
                                .on_press(move || section.set(item))
                                .child(
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .items_start()
                                        .gap(px(8.0))
                                        .child(
                                            image(item.icon())
                                                .size(px(16.0))
                                                .mt(px(1.0))
                                                .tint_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            view()
                                                .flex_col()
                                                .w_full()
                                                .gap(px(1.0))
                                                .flex_grow(1.0)
                                                .min_w(px(0.0))
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
                                                        .w_full()
                                                        .number_of_lines(2)
                                                        .text_size(px(10.0))
                                                        .text_color(move || theme().muted_foreground),
                                                ),
                                        ),
                                )
                        }))
                })),
        )
        .child(
            view()
                .flex_1()
                .min_w(px(320.0))
                .flex_col()
                .gap(px(20.0))
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            text(move || section.get().title())
                                .text_size(px(48.0))
                                .line_height(px(48.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || section.get().subtitle())
                                .text_size(px(16.0))
                                .line_height(px(24.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    when(move || section.get() == Section::Account, move || {
                        account_card(
                            email.clone(),
                            verified,
                            is_admin,
                            name,
                            error,
                            saving,
                            save.clone(),
                            session,
                            delete_open,
                            delete_password,
                            delete_error,
                            data_server,
                            data_token,
                        )
                    })
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::Appearance, move || appearance_card(appearance_server.clone(), appearance_token.clone()))
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
    error: Signal<String>,
    saving: Signal<bool>,
    save: impl Fn() + 'static,
    session: Signal<Session>,
    delete_open: Signal<bool>,
    delete_password: Signal<String>,
    delete_error: Signal<String>,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let initial = email
        .chars()
        .next()
        .map(|value| value.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string());
    let email_value = signal(email.clone());
    view()
        .w_full()
        .flex_col()
        .gap(px(16.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(16.0))
                .child(
                    view()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            text("Personal information")
                                .text_size(px(20.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Your email identifies you; your name is how we greet you.")
                                .text_size(px(14.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .mt(px(32.0))
                        .flex_row()
                        .items_center()
                        .gap(px(12.0))
                        .child(
                            view()
                                .h(px(64.0))
                                .w(px(64.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(999.0))
                                .bg(move || theme().primary)
                                .child(
                                    text(initial)
                                        .text_size(px(22.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().primary_foreground),
                                ),
                        )
                        .child(
                            button()
                                .items_center()
                                .justify_center()
                                .h(px(34.0))
                                .px(px(12.0))
                                .rounded(px(7.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .bg(move || theme().background)
                                .on_press(move || {})
                                .child(
                                    text("Change photo")
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(ui::label("Email"))
                        .child(
                            input()
                                .value(email_value)
                                .readonly(true)
                                .auto_fill(AutoFill::Off)
                                .h(px(40.0))
                                .w_full()
                                .px(px(12.0))
                                .rounded(px(7.0))
                                .border(px(1.0))
                                .border_color(move || theme().input)
                                .bg(move || theme().background)
                                .text_size(px(14.0))
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(ui::field("Name", name, "Your name"))
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
                        .w_full()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.0))
                        .child(
                            button()
                                .items_center()
                                .justify_center()
                                .h(px(40.0))
                                .px(px(16.0))
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
                        ),
                ),
        )
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(6.0))
                .child(
                    text("Delete account")
                        .text_size(px(20.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().destructive),
                )
                .child(
                    text("Permanently remove your account and all of its data.")
                        .text_size(px(13.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            button()
                                .items_center()
                                .justify_center()
                                .h(px(36.0))
                                .px(px(12.0))
                                .rounded(px(7.0))
                                .border(px(1.0))
                                .border_color(move || theme().destructive)
                                .bg(move || theme().destructive.opacity(0.12))
                                .on_press(move || delete_open.set(!delete_open.get()))
                                .child(
                                    text("Delete account")
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().destructive),
                                ),
                        )
                        .when(delete_open.get(), |column| {
                            column
                                .child(
                                    input()
                                        .value(delete_password)
                                        .on_change(move |value: &str| delete_password.set(value.to_string()))
                                        .placeholder("Password")
                                        .masked(true)
                                        .auto_fill(AutoFill::Off)
                                        .h(px(38.0))
                                        .px(px(10.0))
                                        .rounded(px(7.0))
                                        .border(px(1.0))
                                        .border_color(move || theme().input)
                                        .bg(move || theme().background)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text(move || delete_error.get())
                                        .text_size(px(11.0))
                                        .text_color(move || theme().destructive),
                                )
                                .child(
                                    button()
                                        .items_center()
                                        .justify_center()
                                        .h(px(34.0))
                                        .px(px(12.0))
                                        .rounded(px(7.0))
                                        .bg(move || theme().destructive)
                                        .on_press(move || {
                                            let password = delete_password.get();
                                            if password.is_empty() {
                                                delete_error.set("Enter your password.".to_string());
                                                return;
                                            }
                                            let server_value = server.get();
                                            let token_value = token.get();
                                            spawn(async move {
                                                match crate::api::delete_account(server_value, token_value, password).await {
                                                    Ok(()) => {
                                                        auth::clear_session();
                                                        session.set(Session::SignedOut);
                                                    }
                                                    Err(message) => delete_error.set(message),
                                                }
                                            });
                                        })
                                        .child(
                                            text("Confirm deletion")
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().primary_foreground),
                                        ),
                                )
                        }),
            ),
        )
}

fn appearance_card(server: String, token: String) -> impl IntoElement {
    let selected = signal(crate::theme::saved());
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(14.0))
                .child(
                    view()
                        .flex_col()
                        .gap(px(3.0))
                        .child(
                            text("Appearance")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || {
                                crate::theme::theme_by_id(&selected.get())
                                    .map_or_else(|| "Choose a palette".to_string(), |theme| format!("{} · {}", theme.label, theme.description))
                            })
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(2, minmax(0, 1fr))"))
                        .gap(px(8.0))
                        .children(crate::theme::THEMES.iter().map(|definition| {
                            let id = definition.id;
                            let card_server = server.clone();
                            let card_token = token.clone();
                            let active = move || selected.get() == id;
                            button()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .gap(px(10.0))
                                .p(px(10.0))
                                .rounded(px(9.0))
                                .border(px(1.0))
                                .border_color(move || if active() { theme().primary } else { theme().border })
                                .bg(move || if active() { theme().accent } else { theme().background })
                                .on_press(move || {
                                    crate::theme::apply(id);
                                    crate::theme::save(id);
                                    selected.set(id.to_string());
                                    let server = card_server.clone();
                                    let token = card_token.clone();
                                    spawn(async move {
                                        let _ = crate::api::save_theme(server, token, id).await;
                                    });
                                })
                                .child(
                                    text(definition.label)
                                        .flex_grow(1.0)
                                        .number_of_lines(1)
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(3.0))
                                        .child(view().h(px(12.0)).w(px(12.0)).rounded(px(3.0)).bg(rgba((definition.background << 8) | 0xff)))
                                        .child(view().h(px(12.0)).w(px(12.0)).rounded(px(3.0)).bg(rgba((definition.card << 8) | 0xff)))
                                        .child(view().h(px(12.0)).w(px(12.0)).rounded(px(3.0)).bg(rgba((definition.primary << 8) | 0xff))),
                                )
                                .child(
                                    when(active, move || {
                                        image(asset!("icons/check.svg")).size(px(13.0)).tint_color(move || theme().primary).into_element()
                                    })
                                    .otherwise(|| view().h(px(0.0)).into_element()),
                                )
                                .into_element()
                        })),
                ),
        )
}

fn general_card(currency: Signal<String>, save: impl Fn() + 'static) -> impl IntoElement {
    ui::surface()
        .w_full()
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
                .items_center()
                .justify_center()
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
        .w_full()
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
                                    .child(shared_toggle(enabled, channel_label(&channel), move |_| toggle()))
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
        .w_full()
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
                                .items_center()
                                .justify_center()
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
    let account_name = signal(String::new());
    let account_edit = signal(None::<Uuid>);
    let account_open = signal(false);
    let account_error = signal(String::new());
    let account_busy = signal(false);
    let category_name = signal(String::new());
    let category_edit = signal(None::<Uuid>);
    let category_open = signal(false);
    let category_error = signal(String::new());
    let category_busy = signal(false);

    let start_account = {
        move || {
            account_name.set(String::new());
            account_edit.set(None);
            account_error.set(String::new());
            account_open.set(true);
        }
    };
    let save_account = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let name = account_name.get().trim().to_string();
            if name.is_empty() {
                account_error.set("Enter an account name.".to_string());
                return;
            }
            let editing = account_edit.get();
            let server = server.clone();
            let token = token.clone();
            account_busy.set(true);
            spawn(async move {
                let result = match editing {
                    Some(id) => crate::api::update_account(server, token, id, &name).await.map(|_| ()),
                    None => crate::api::create_account(server, token, &name).await.map(|_| ()),
                };
                match result {
                    Ok(()) => {
                        account_open.set(false);
                        account_error.set(String::new());
                        data.refetch();
                    }
                    Err(message) => account_error.set(message),
                }
                account_busy.set(false);
            });
        }
    };
    let start_category = {
        move || {
            category_name.set(String::new());
            category_edit.set(None);
            category_error.set(String::new());
            category_open.set(true);
        }
    };
    let save_category = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let name = category_name.get().trim().to_string();
            if name.is_empty() {
                category_error.set("Enter a category name.".to_string());
                return;
            }
            let editing = category_edit.get();
            let server = server.clone();
            let token = token.clone();
            category_busy.set(true);
            spawn(async move {
                let result = match editing {
                    Some(id) => crate::api::update_category(server, token, id, &name).await.map(|_| ()),
                    None => crate::api::create_category(server, token, &name).await.map(|_| ()),
                };
                match result {
                    Ok(()) => {
                        category_open.set(false);
                        category_error.set(String::new());
                        data.refetch();
                    }
                    Err(message) => category_error.set(message),
                }
                category_busy.set(false);
            });
        }
    };

    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(14.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text("Accounts")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(ui::action_button("Add account", false, start_account)),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || data.get().map_or_else(Vec::new, |(accounts, _)| accounts),
                            |item| item.account.id,
                            {
                                let delete_server = server.clone();
                                let delete_token = token.clone();
                                move |item, _| {
                                    let item: crate::models::AccountWithCounts = item.get();
                                    let id = item.account.id;
                                    let row_name = item.account.name.clone();
                                    let edit_name = row_name.clone();
                                    let delete_server = delete_server.clone();
                                    let delete_token = delete_token.clone();
                                    let delete = move || {
                                        let server = delete_server.clone();
                                        let token = delete_token.clone();
                                        spawn(async move {
                                            match crate::api::delete_account_by_id(server, token, id).await {
                                                Ok(()) => data.refetch(),
                                                Err(message) => account_error.set(message),
                                            }
                                        });
                                    };
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .py(px(7.0))
                                        .border_b(px(1.0))
                                        .border_color(move || theme().border)
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(7.0))
                                                .child(
                                                    text(row_name)
                                                        .text_size(px(13.0))
                                                        .text_color(move || theme().foreground),
                                                )
                                                .when(item.account.is_default, |row| {
                                                    row.child(text("Default").text_size(px(9.0)).font_weight(FontWeight::SEMIBOLD).text_color(move || theme().primary))
                                                }),
                                        )
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    text(format!(
                                                        "{} subscriptions · {} transactions",
                                                        item.subscription_count, item.transaction_count
                                                    ))
                                                    .text_size(px(10.0))
                                                    .text_color(move || theme().muted_foreground),
                                                )
                                                .child(ui::mini_icon_button(asset!("icons/pencil.svg"), false, move || {
                                                    account_name.set(edit_name.clone());
                                                    account_edit.set(Some(id));
                                                    account_error.set(String::new());
                                                    account_open.set(true);
                                                }))
                                                .child(ui::mini_icon_button(asset!("icons/trash.svg"), true, delete)),
                                        )
                                }
                            },
                        )),
                )
                .child(
                    when(move || account_open.get(), {
                    let save = save_account.clone();
                    move || {
                        let save = save.clone();
                        ui::surface()
                            .p(px(14.0))
                            .gap(px(10.0))
                            .border_color(move || theme().primary.opacity(0.4))
                            .bg(move || theme().muted.opacity(0.4))
                            .child(
                                text(move || if account_edit.get().is_some() { "Edit account" } else { "New account" })
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(ui::field("Name", account_name, "Main account"))
                            .child(
                                when(move || !account_error.get().is_empty(), move || {
                                    text(move || account_error.get())
                                        .text_size(px(11.0))
                                        .text_color(move || theme().destructive)
                                        .into_element()
                                })
                                .otherwise(|| view().h(px(0.0)).into_element()),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .child(ui::action_button("Cancel", false, move || account_open.set(false)))
                                    .child(ui::action_button(
                                        if account_busy.get() { "Saving..." } else { "Save" },
                                        false,
                                        save,
                                    )),
                            )
                            .into_element()
                    }
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    when(move || !account_error.get().is_empty() && !account_open.get(), move || {
                        text(move || account_error.get())
                            .text_size(px(11.0))
                            .text_color(move || theme().destructive)
                            .into_element()
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    text("Accounts are required. Every subscription and transaction belongs to one.")
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(14.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text("Categories")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(ui::action_button("Add category", false, start_category)),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || data.get().map_or_else(Vec::new, |(_, categories)| categories),
                            |item| item.category.id,
                            {
                                let delete_server = server.clone();
                                let delete_token = token.clone();
                                move |item, _| {
                                    let item: crate::models::CategoryWithCounts = item.get();
                                    let id = item.category.id;
                                    let row_name = item.category.name.clone();
                                    let edit_name = row_name.clone();
                                    let delete_server = delete_server.clone();
                                    let delete_token = delete_token.clone();
                                    let delete = move || {
                                        let server = delete_server.clone();
                                        let token = delete_token.clone();
                                        spawn(async move {
                                            match crate::api::delete_category_by_id(server, token, id).await {
                                                Ok(()) => data.refetch(),
                                                Err(message) => category_error.set(message),
                                            }
                                        });
                                    };
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .py(px(7.0))
                                        .border_b(px(1.0))
                                        .border_color(move || theme().border)
                                        .child(
                                            text(row_name)
                                                .text_size(px(13.0))
                                                .text_color(move || theme().foreground),
                                        )
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    text(format!(
                                                        "{} subscriptions · {} transactions",
                                                        item.subscription_count, item.transaction_count
                                                    ))
                                                    .text_size(px(10.0))
                                                    .text_color(move || theme().muted_foreground),
                                                )
                                                .child(ui::mini_icon_button(asset!("icons/pencil.svg"), false, move || {
                                                    category_name.set(edit_name.clone());
                                                    category_edit.set(Some(id));
                                                    category_error.set(String::new());
                                                    category_open.set(true);
                                                }))
                                                .child(ui::mini_icon_button(asset!("icons/trash.svg"), true, delete)),
                                        )
                                }
                            },
                        )),
                )
                .child(
                    when(move || category_open.get(), {
                    let save = save_category.clone();
                    move || {
                        let save = save.clone();
                        ui::surface()
                            .p(px(14.0))
                            .gap(px(10.0))
                            .border_color(move || theme().primary.opacity(0.4))
                            .bg(move || theme().muted.opacity(0.4))
                            .child(
                                text(move || if category_edit.get().is_some() { "Edit category" } else { "New category" })
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(ui::field("Name", category_name, "Entertainment"))
                            .child(
                                when(move || !category_error.get().is_empty(), move || {
                                    text(move || category_error.get())
                                        .text_size(px(11.0))
                                        .text_color(move || theme().destructive)
                                        .into_element()
                                })
                                .otherwise(|| view().h(px(0.0)).into_element()),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .child(ui::action_button("Cancel", false, move || category_open.set(false)))
                                    .child(ui::action_button(
                                        if category_busy.get() { "Saving..." } else { "Save" },
                                        false,
                                        save,
                                    )),
                            )
                            .into_element()
                    }
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    when(move || !category_error.get().is_empty() && !category_open.get(), move || {
                        text(move || category_error.get())
                            .text_size(px(11.0))
                            .text_color(move || theme().destructive)
                            .into_element()
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    text("Categories are optional. Subscriptions and transactions can be uncategorized.")
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
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
    let settings_data = resource(
        || Some(()),
        {
            let server = server.clone();
            let token = token.clone();
            move |(), _| {
                let server = server.clone();
                let token = token.clone();
                async move { crate::api::load_admin_settings(server, token).await }
            }
        },
    );
    let registration = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            let enabled = settings_data
                .get()
                .map_or(false, |settings: crate::api::AdminSettings| settings.registration_enabled);
            spawn(async move {
                let _ = crate::api::update_admin_settings(server, token, !enabled).await;
                settings_data.refetch();
            });
        }
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(24.0))
                .gap(px(12.0))
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
                                    text("Registration")
                                        .text_size(px(13.0))
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text("Allow new accounts to sign up.")
                                        .text_size(px(10.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(shared_toggle(
                            settings_data
                                .get()
                                .map_or(false, |settings: crate::api::AdminSettings| settings.registration_enabled),
                            "Registration".to_string(),
                            move |_| registration(),
                        )),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(6.0))
                        .child(each(
                            move || data.get().unwrap_or_default(),
                            |user| user.id,
                            {
                                let role_server = server.clone();
                                let role_token = token.clone();
                                let delete_server = server.clone();
                                let delete_token = token.clone();
                                move |user, _| {
                                    let user: crate::models::AdminUser = user.get();
                                    let id = user.id;
                                    let is_admin = user.is_admin;
                                    let row_name = user.name.clone().unwrap_or_else(|| user.email.clone());
                                    let row_email = user.email.clone();
                                    let role_server = role_server.clone();
                                    let role_token = role_token.clone();
                                    let role = move || {
                                        let server = role_server.clone();
                                        let token = role_token.clone();
                                        spawn(async move {
                                            let _ = crate::api::update_admin_role(server, token, id, !is_admin).await;
                                            data.refetch();
                                        });
                                    };
                                    let delete_server = delete_server.clone();
                                    let delete_token = delete_token.clone();
                                    let delete = move || {
                                        let server = delete_server.clone();
                                        let token = delete_token.clone();
                                        spawn(async move {
                                            let _ = crate::api::delete_admin_user(server, token, id).await;
                                            data.refetch();
                                        });
                                    };
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .py(px(7.0))
                                        .border_b(px(1.0))
                                        .border_color(move || theme().border)
                                        .child(
                                            view()
                                                .flex_col()
                                                .child(
                                                    text(row_name)
                                                        .text_size(px(13.0))
                                                        .text_color(move || theme().foreground),
                                                )
                                                .child(
                                                    text(row_email)
                                                        .text_size(px(10.0))
                                                        .text_color(move || theme().muted_foreground),
                                                ),
                                        )
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    text(if is_admin { "Admin" } else { "User" })
                                                        .text_size(px(10.0))
                                                        .text_color(move || theme().muted_foreground),
                                                )
                                                .child(ui::action_button(
                                                    if is_admin { "Remove admin" } else { "Make admin" },
                                                    false,
                                                    role,
                                                ))
                                                .child(ui::mini_icon_button(asset!("icons/trash.svg"), true, delete)),
                                        )
                                }
                            },
                        )),
                ),
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
