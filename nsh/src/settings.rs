//! Settings: account, appearance, server and about.

use crate::models::AiSettings;
use base64::Engine;
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::file_dialog::OpenDialog;
#[cfg(not(target_arch = "wasm32"))]
use nsh_core::file_dialog::{FileFilter, SaveDialog};
use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::{self, AuthSession, AuthUser, Session};
use crate::components::badge::{BadgeProps, BadgeTone, badge};
use crate::components::checkbox::checkbox;
use crate::components::string_select::{StringSelectProps, string_select};
use crate::components::toggle::toggle as shared_toggle;
use crate::ui;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SettingsResponse {
    pub user: AuthUser,
}

#[derive(Debug, Serialize)]
pub struct UpdateSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
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
            Section::Admin => asset!("icons/shield.svg"),
            Section::About => asset!("icons/info.svg"),
        }
    }

    fn title(self) -> &'static str {
        self.label()
    }

    fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "account" => Self::Account,
            "appearance" => Self::Appearance,
            "general" => Self::General,
            "ai" => Self::Ai,
            "money" => Self::Money,
            "notifications" => Self::Notifications,
            "data" => Self::Data,
            "server" => Self::Server,
            "admin" => Self::Admin,
            "about" => Self::About,
            _ => return None,
        })
    }

    fn subtitle(self) -> &'static str {
        match self {
            Section::Account => "Your profile, session and account",
            Section::Appearance => "Color themes and palette",
            Section::General => "Defaults, locale, exchange rates and app updates",
            Section::Notifications => "Choose where your summary lands and when it leaves",
            Section::Ai => "Model, voice input and privacy",
            Section::Money => "Accounts and categories that organize your money",
            Section::Data => "Keep a copy of your data and sync devices",
            Section::Server => "Server connection and device pairing",
            Section::Admin => "Users, registration and full installation backups",
            Section::About => "Recurrency, the people behind it, and what changed",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Account => "Profile, session and account removal",
            Section::Appearance => "Color themes and palette",
            Section::General => "Currency, exchange rates, week start and dates",
            Section::Notifications => "Delivery channels and summaries",
            Section::Ai => "Model, voice input and privacy",
            Section::Money => "Accounts and categories",
            Section::Data => "Backups, restore and folder sync",
            Section::Server => "Server connection and device pairing",
            Section::Admin => "Users, registration and full backups",
            Section::About => "Version, release notes and Android app",
        }
    }
}

fn compact_nav(section: Signal<Section>, is_admin: bool) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .gap(px(4.0))
        .children(
            Section::ALL
                .iter()
                .filter(|item| {
                    !crate::local::enabled()
                        || matches!(
                            item,
                            Section::Account
                                | Section::Appearance
                                | Section::General
                                | Section::Money
                                | Section::Data
                                | Section::About
                        )
                })
                .filter(|item| **item != Section::Admin || is_admin)
                .map(|item| {
                    let item = *item;
                    let active = move || section.get() == item;
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(32.0))
                        .px(px(10.0))
                        .rounded(px(7.0))
                        .bg(move || {
                            if active() {
                                theme().accent
                            } else {
                                theme().background
                            }
                        })
                        .border(px(1.0))
                        .border_color(move || {
                            if active() {
                                theme().primary
                            } else {
                                theme().border
                            }
                        })
                        .on_press(move || section.set(item))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(5.0))
                                .child(
                                    image(item.icon())
                                        .size(px(13.0))
                                        .tint_color(move || theme().muted_foreground),
                                )
                                .child(
                                    text(item.label())
                                        .text_size(px(11.0))
                                        .text_color(move || theme().foreground),
                                ),
                        )
                        .into_element()
                }),
        )
}

pub fn settings(auth: AuthSession, session: Signal<Session>) -> AnyElement {
    let section = signal(Section::Account);
    effect(move || {
        if let Some(key) = crate::ui::settings_intent() {
            if let Some(requested) = Section::from_key(&key) {
                if !crate::local::enabled()
                    || matches!(
                        requested,
                        Section::Account
                            | Section::Appearance
                            | Section::General
                            | Section::Money
                            | Section::Data
                            | Section::About
                    )
                {
                    section.set(requested);
                }
            }
            crate::ui::clear_settings_intent();
        }
    });
    let name = signal(auth.user.name.clone().unwrap_or_default());
    let avatar = signal(auth.user.avatar.clone().unwrap_or_default());
    let email_signal = signal(auth.user.email.clone());
    let currency = signal(auth.user.currency.clone());
    let error = signal(String::new());
    let saving = signal(false);
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let verified = auth.user.verified;
    let is_admin = auth.user.is_admin;
    let backup_status = signal(String::new());
    let modal = ui::ModalHost::new();
    let delete_password = signal(String::new());
    let delete_error = signal(String::new());
    let save_server = server.clone();
    let save_token = token.clone();
    let data_server = signal(server.clone());
    let data_token = signal(token.clone());
    let info_server = server.clone();
    let appearance_server = server.clone();
    let appearance_token = token.clone();
    let general_server = server.clone();
    let general_token = token.clone();

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
                email: Some(email_signal.get().trim().to_string()),
                avatar: Some(avatar.get()),
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
                        crate::components::toast::success("Profile saved.");
                        saving.set(false);
                        session.set(Session::SignedIn(next));
                    }
                    Err(message) => {
                        crate::components::toast::error(message.clone());
                        error.set(message);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_general = save.clone();
    let compact = signal(false);
    view()
        .relative()
        .w_full()
        .pt(px(11.0))
        .on_layout(
            move |bounds: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
                compact.set(bounds.size.width.0 < 900.0);
            },
        )
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(px(0.0))
        .child(when(
            move || compact.get(),
            move || compact_nav(section, is_admin),
        ))
        .child(
            view()
                .w(move || if compact.get() { px(0.0) } else { px(208.0) })
                .mr(move || if compact.get() { px(0.0) } else { px(40.0) })
                .max_h(move || if compact.get() { px(0.0) } else { px(4000.0) })
                .overflow_hidden()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text("SETTINGS")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .mb(px(10.0)),
                )
                .children(
                    [
                        ("PERSONAL", &[Section::Account, Section::Appearance][..]),
                        (
                            "CONFIGURATION",
                            &[
                                Section::General,
                                Section::Ai,
                                Section::Money,
                                Section::Notifications,
                            ][..],
                        ),
                        (
                            "SYSTEM",
                            &[
                                Section::Data,
                                Section::Server,
                                Section::Admin,
                                Section::About,
                            ][..],
                        ),
                    ]
                    .into_iter()
                    .map(|(group, items)| {
                        view()
                            .flex_col()
                            .gap(px(9.0))
                            .mb(if group == "PERSONAL" {
                                px(15.0)
                            } else {
                                px(2.0)
                            })
                            .child(
                                text(group)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().muted_foreground)
                                    .mb(px(2.0)),
                            )
                            .children(
                                items
                                    .iter()
                                    .filter(|item| {
                                        **item != Section::Admin
                                            || (is_admin && !crate::local::enabled())
                                    })
                                    .map(|item| {
                                        let item = *item;
                                        let active = move || section.get() == item;
                                        button()
                                            .cursor_pointer()
                                            .items_center()
                                            .justify_start()
                                            .w_full()
                                            .rounded(px(8.0))
                                            .px(px(18.0))
                                            .py(px(10.0))
                                            .bg(move || {
                                                if active() {
                                                    theme().accent
                                                } else {
                                                    theme().background
                                                }
                                            })
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
                                                            .tint_color(move || {
                                                                theme().muted_foreground
                                                            }),
                                                    )
                                                    .child(
                                                        view()
                                                            .flex_col()
                                                            .w_full()
                                                            .gap(px(2.0))
                                                            .flex_grow(1.0)
                                                            .min_w(px(0.0))
                                                            .child(
                                                                text(item.label())
                                                                    .text_size(px(14.0))
                                                                    .font_weight(FontWeight::MEDIUM)
                                                                    .text_color(move || {
                                                                        if active() {
                                                                            theme().foreground
                                                                        } else {
                                                                            theme().muted_foreground
                                                                        }
                                                                    }),
                                                            )
                                                            .child(
                                                                text(item.description())
                                                                    .w_full()
                                                                    .number_of_lines(2)
                                                                    .text_size(px(12.0))
                                                                    .text_color(move || {
                                                                        theme().muted_foreground
                                                                    }),
                                                            ),
                                                    ),
                                            )
                                    }),
                            )
                    }),
                )
                .child(
                    view().w_full().mt(px(17.0)).child(
                        button()
                            .cursor_pointer()
                            .w_full()
                            .h(px(40.0))
                            .px(px(18.0))
                            .rounded(px(6.0))
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .on_press(move || {
                                crate::auth::clear_session();
                                session.set(Session::SignedOut);
                            })
                            .child(
                                image(asset!("icons/logout.svg"))
                                    .size(px(16.0))
                                    .tint_color(move || theme().muted_foreground),
                            )
                            .child(
                                text("Log out")
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().muted_foreground),
                            ),
                    ),
                ),
        )
        .child(
            view()
                .flex_1()
                .min_w(px(320.0))
                .flex_col()
                .mt(px(0.0))
                .gap(px(0.0))
                .child(
                    when(
                        move || !matches!(section.get(), Section::Money | Section::Ai),
                        move || {
                            view()
                                .flex_col()
                                .gap(px(0.0))
                                .child(
                                    text(move || section.get().title())
                                        .text_size(
                                            move || if compact.get() { px(36.0) } else { px(48.0) },
                                        )
                                        .line_height(move || {
                                            if compact.get() { px(40.0) } else { px(48.0) }
                                        })
                                        .letter_spacing(px(-2.4))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text(move || section.get().subtitle())
                                        .text_size(px(16.0))
                                        .line_height(px(24.0))
                                        .text_color(move || theme().muted_foreground),
                                )
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || !matches!(section.get(), Section::Money | Section::Ai),
                        || view().h(px(24.0)),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Account,
                        move || {
                            account_card(
                                email_signal,
                                verified,
                                is_admin,
                                name,
                                avatar,
                                error,
                                saving,
                                save.clone(),
                                session,
                                modal,
                                delete_password,
                                delete_error,
                                data_server,
                                data_token,
                            )
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Appearance,
                        move || {
                            appearance_card(appearance_server.clone(), appearance_token.clone())
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::General,
                        move || {
                            general_card(
                                currency,
                                save_general.clone(),
                                general_server.clone(),
                                general_token.clone(),
                            )
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Notifications,
                        move || notifications_card(data_server.get(), data_token.get()),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Ai,
                        move || ai_card(data_server.get(), data_token.get()),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Money,
                        move || money_card(data_server.get(), data_token.get(), modal),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Admin,
                        move || admin_card(data_server.get(), data_token.get(), modal),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Data,
                        move || {
                            view()
                                .w_full()
                                .flex_col()
                                .gap(px(12.0))
                                .child(data_card(
                                    data_server.get(),
                                    data_token.get(),
                                    backup_status,
                                    false,
                                ))
                                .child(folder_sync_card(data_server.get(), data_token.get()))
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || section.get() == Section::Server,
                        move || server_card(info_server.clone(), session),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(move || section.get() == Section::About, || about_card())
                        .otherwise(|| view().h(px(0.0))),
                ),
        )
        .child(
            when(move || modal.active(), move || ui::modal_overlay(modal))
                .otherwise(|| view().h(px(0.0))),
        )
        .into_element()
}

async fn save_backup(name: &str, json: &str, status: Signal<String>) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::web_files::download_text(name, "application/json", json) {
            Ok(()) => status.set("Backup downloaded.".to_string()),
            Err(error) => status.set(error),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match pick_save_path(
            SaveDialog::new()
                .file_name(name)
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
        }
    }
}

async fn pick_backup_text() -> Result<Option<String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::web_files::pick_text_file(".json,application/json").await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match pick_paths(
            OpenDialog::files()
                .title("Restore Recurrency backup")
                .filter(FileFilter::new("JSON", ["json"])),
        )
        .await?
        {
            Some(paths) => match paths.first() {
                Some(path) => std::fs::read_to_string(path)
                    .map(Some)
                    .map_err(|error| format!("Could not read the backup: {error}")),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}

pub(crate) async fn pick_image_data_url() -> Result<Option<String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(data_url) =
            crate::web_files::pick_data_url("image/png,image/jpeg,image/webp").await?
        else {
            return Ok(None);
        };
        let encoded = data_url
            .split_once(";base64,")
            .map(|(_, encoded)| encoded)
            .ok_or_else(|| "Could not read the image.".to_string())?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| "Could not read the image.".to_string())?;
        avatar_data_url(&bytes).map(Some)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match pick_paths(
            OpenDialog::files()
                .title("Choose an image")
                .filter(FileFilter::new("Images", ["png", "jpg", "jpeg", "webp"])),
        )
        .await?
        {
            Some(paths) => match paths.first() {
                Some(path) => std::fs::read(path)
                    .map_err(|error| format!("Could not read the image: {error}"))
                    .and_then(|bytes| avatar_data_url(&bytes))
                    .map(Some),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}

pub(crate) fn avatar_data_url(bytes: &[u8]) -> Result<String, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|_| "Could not process that image. Try a PNG or JPEG.".to_string())?;
    let resized = image
        .resize(256, 256, image::imageops::FilterType::Lanczos3)
        .to_rgb8();
    let mut output = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 86)
        .encode_image(&resized)
        .map_err(|_| "Could not process that image. Try a PNG or JPEG.".to_string())?;
    Ok(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(output)
    ))
}

fn account_card(
    email: Signal<String>,
    _verified: bool,
    _is_admin: bool,
    name: Signal<String>,
    avatar: Signal<String>,
    error: Signal<String>,
    saving: Signal<bool>,
    save: impl Fn() + 'static,
    session: Signal<Session>,
    modal: ui::ModalHost,
    delete_password: Signal<String>,
    delete_error: Signal<String>,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let pick_avatar = {
        let server = server;
        let token = token;
        let avatar = avatar;
        move || {
            let server = server.get();
            let session_token = token.get();
            let token = session_token.clone();
            let avatar = avatar;
            spawn(async move {
                match pick_image_data_url().await {
                    Ok(Some(data_url)) => {
                        let request = UpdateSettingsRequest {
                            name: None,
                            email: None,
                            avatar: Some(data_url.clone()),
                            currency: None,
                        };
                        match crate::api::update_settings(server, token, &request).await {
                            Ok(response) => {
                                let next = AuthSession {
                                    token: session_token.clone(),
                                    user: response.user,
                                    requires_verification: false,
                                };
                                crate::auth::store_session(&next);
                                avatar.set(data_url);
                                session.set(Session::SignedIn(next));
                            }
                            Err(message) => error.set(message),
                        }
                    }
                    Ok(None) => {}
                    Err(problem) => error.set(problem),
                }
            });
        }
    };
    let open_delete = {
        let server = server;
        let token = token;
        let session = session;
        let delete_password = delete_password;
        let delete_error = delete_error;
        move || {
            delete_error.set(String::new());
            modal.open(move || {
                let confirm_server = server.get();
                let confirm_token = token.get();
                crate::components::dialog::dialog(
                    crate::components::dialog::DialogProps::new(
                        "Delete account",
                        "This permanently removes your account and all of its data.",
                        move || modal.close(),
                    )
                    .width(420.0)
                    .body(
                        view()
                            .p(px(24.0))
                            .gap(px(10.0))
                            .child(ui::label("Password"))
                            .child(
                                input()
                                    .value(delete_password)
                                    .on_change(move |value: &str| {
                                        delete_password.set(value.to_string())
                                    })
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
                            .into_element(),
                    )
                    .footer(
                        view()
                            .w_full()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(ui::action_button("Cancel", false, move || modal.close()))
                            .child(ui::action_button("Confirm deletion", true, move || {
                                let password = delete_password.get();
                                if password.is_empty() {
                                    delete_error.set("Enter your password.".to_string());
                                    return;
                                }
                                let server = confirm_server.clone();
                                let token = confirm_token.clone();
                                spawn(async move {
                                    match crate::api::delete_account(server, token, password).await
                                    {
                                        Ok(()) => {
                                            modal.close();
                                            auth::clear_session();
                                            session.set(Session::SignedOut);
                                        }
                                        Err(message) => delete_error.set(message),
                                    }
                                });
                            }))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(
            view()
                .w_full()
                .rounded(px(12.0))
                .border(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .bg(move || theme().card.opacity(0.8))
                .flex_col()
                .child(
                    view()
                        .p(px(24.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            text("Personal information")
                                .text_size(px(18.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Your email identifies you; your name is how we greet you.")
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .px(px(49.0))
                        .pt(px(24.0))
                        .pb(px(45.0))
                        .gap(px(16.0))
                        .flex_col()
                        .child(
                            view()
                                .mt(px(24.0))
                                .flex_row()
                                .items_center()
                                .gap(px(16.0))
                                .child(crate::components::avatar::avatar_badge(
                                    64.0,
                                    &name.get(),
                                    &email.get(),
                                    (!avatar.get().is_empty()).then(|| avatar.get()),
                                ))
                                .child(
                                    button()
                                        .cursor_pointer()
                                        .items_center()
                                        .justify_center()
                                        .h(px(36.0))
                                        .w(px(110.0))
                                        .px(px(11.0))
                                        .rounded(px(6.0))
                                        .border(px(1.0))
                                        .border_color(move || theme().border)
                                        .bg(move || theme().background)
                                        .on_press(move || pick_avatar())
                                        .child(
                                            text("Change photo")
                                                .text_size(px(14.0))
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
                                        .value(email)
                                        .on_change(move |next: &str| email.set(next.to_string()))
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
                            view().w_full().flex_row().justify_end().gap(px(8.0)).child(
                                button()
                                    .cursor_pointer()
                                    .items_center()
                                    .justify_center()
                                    .h(px(40.0))
                                    .min_w(px(115.0))
                                    .px(px(16.0))
                                    .rounded(px(6.0))
                                    .bg(move || theme().primary)
                                    .disabled(move || saving.get())
                                    .disabled_style(|state| state.opacity(0.5))
                                    .on_press(move || save())
                                    .child(
                                        text(move || {
                                            if saving.get() {
                                                "Saving..."
                                            } else {
                                                "Save changes"
                                            }
                                        })
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().primary_foreground),
                                    ),
                            ),
                        ),
                ),
        )
        .child(
            ui::surface()
                .border_color(move || theme().destructive.opacity(0.3))
                .p(px(24.0))
                .pb(px(35.0))
                .gap(px(12.0))
                .child(
                    text("Delete account")
                        .text_size(px(18.0))
                        .line_height(px(28.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().destructive),
                )
                .child(
                    text("Permanently erase your account and all of its data")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(12.0))
                        .pr(px(25.0))
                        .mt(px(56.0))
                        .child(
                            view()
                                .flex_col()
                                .min_w(px(0.0))
                                .child(
                                    text(move || email.get())
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text("This cannot be undone.")
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .items_center()
                                .justify_center()
                                .h(px(40.0))
                                .min_w(px(126.0))
                                .px(px(14.0))
                                .rounded(px(6.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .bg(move || theme().background)
                                .on_press(move || open_delete())
                                .child(
                                    text("Delete account")
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().destructive),
                                ),
                        ),
                ),
        )
}

fn settings_section(
    title: &'static str,
    description: &'static str,
    content: AnyElement,
) -> AnyElement {
    ui::surface()
        .p(px(0.0))
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(4.0))
                .px(px(24.0))
                .pt(px(19.0))
                .pb(px(25.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .child(
                    text(title)
                        .text_size(px(18.0))
                        .line_height(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(description)
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .child(view().w_full().p(px(24.0)).child(content)),
        )
        .into_element()
}

fn appearance_card(server: String, token: String) -> impl IntoElement {
    let selected = signal(crate::theme::saved());
    settings_section(
        "Color themes",
        "Choose a palette. It applies immediately and syncs across devices when connected.",
        crate::components::theme_picker::theme_picker(selected, server, token).into_element(),
    )
}

fn general_card(
    currency: Signal<String>,
    save: impl Fn() + 'static,
    server: String,
    token: String,
) -> impl IntoElement {
    let date_format = signal(crate::format::date_format());
    let first_day = signal(crate::format::first_day());
    let show_updates = signal(crate::update_banner::show_updates());
    effect(move || crate::update_banner::set_show_updates(show_updates.get()));
    let first_server = server.clone();
    let first_token = token.clone();
    let rates_server = server.clone();
    let rates_token = token.clone();
    effect(move || {
        let value = first_day.get();
        crate::format::set_first_day(&value);
        let server = first_server.clone();
        let token = first_token.clone();
        spawn(async move {
            let _ = crate::api::save_first_day(server, token, &value).await;
        });
    });
    effect(move || {
        let value = date_format.get();
        crate::format::set_date_format(&value);
        let server = server.clone();
        let token = token.clone();
        spawn(async move {
            let _ = crate::api::save_date_format(server, token, &value).await;
        });
    });
    let last_currency = signal(currency.get());
    effect(move || {
        let value = currency.get();
        if value != last_currency.get() {
            last_currency.set(value);
            save();
        }
    });
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(settings_section(
            "Defaults",
            "Applied to new entries and reports",
            view()
                .w_full()
                .flex_col()
                .gap(px(12.0))
                .child(string_select(StringSelectProps::new(
                    "Default Currency",
                    currency,
                    crate::currencies::CURRENCIES
                        .iter()
                        .map(|(code, label)| ((*code).to_string(), (*label).to_string()))
                        .collect(),
                )))
                .child(string_select(StringSelectProps::new(
                    "First Day of Week",
                    first_day,
                    vec![
                        ("monday".to_string(), "Monday".to_string()),
                        ("sunday".to_string(), "Sunday".to_string()),
                    ],
                )))
                .child(string_select(StringSelectProps::new(
                    "Date format",
                    date_format,
                    vec![
                        ("DD/MM/YYYY".to_string(), "DD/MM/YYYY".to_string()),
                        ("MM/DD/YYYY".to_string(), "MM/DD/YYYY".to_string()),
                        ("YYYY/MM/DD".to_string(), "YYYY/MM/DD".to_string()),
                    ],
                )))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_start()
                        .gap(px(12.0))
                        .p(px(14.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.4))
                        .child(checkbox(show_updates, String::new()))
                        .child(
                            view()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    text("Show update notices")
                                        .text_size(px(14.0))
                                        .line_height(px(20.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text("Check for new Recurrency releases and show a dismissible banner.")
                                        .text_size(px(12.0))
                                        .line_height(px(16.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .into_element(),
        ))
        .when(!crate::local::enabled(), |view| {
            view.child(exchange_rates_card(rates_server, rates_token))
        })
}

fn folder_sync_card(server: String, token: String) -> AnyElement {
    if cfg!(target_arch = "wasm32") {
        return view().h(px(0.0)).into_element();
    }
    let folder = signal(crate::folder_sync::folder());
    let last_sync = signal(crate::folder_sync::last_sync());
    let status = signal(String::new());
    let choose = move || {
        spawn(async move {
            match pick_paths(
                OpenDialog::folders()
                    .title("Choose a sync folder")
                    .button_label("Use folder"),
            )
            .await
            {
                Ok(Some(paths)) => {
                    let Some(path) = paths.first() else { return };
                    crate::folder_sync::set_folder(path);
                    folder.set(Some(path.clone()));
                    status.set("Sync folder connected.".to_string());
                }
                Ok(None) => {}
                Err(error) => status.set(error),
            }
        });
    };
    let sync_now = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::folder_sync::sync(server, token).await {
                    Ok(path) => {
                        last_sync.set(crate::folder_sync::last_sync());
                        crate::components::toast::success("Backup synced to the folder.");
                        status.set(format!("Backup written to {}", path.display()));
                    }
                    Err(error) => {
                        crate::components::toast::error(error.clone());
                        status.set(error);
                    }
                }
            });
        }
    };
    let disconnect = move || {
        crate::folder_sync::disconnect();
        folder.set(None);
        last_sync.set(String::new());
        crate::components::toast::success("Sync folder disconnected.");
        status.set("Sync folder disconnected.".to_string());
    };
    ui::surface()
        .p(px(24.0))
        .gap(px(10.0))
        .child(text("Cloud folder sync").text_size(px(18.0)).font_weight(FontWeight::BOLD).text_color(move || theme().foreground))
        .child(
            text("Write a validated backup into a folder managed by Drive, OneDrive, Dropbox or another document provider.")
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .p(px(12.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .child(image(asset!("icons/database.svg")).size(px(20.0)).tint_color(move || theme().primary))
                .child(
                    view()
                        .flex_col()
                        .flex_grow(1.0)
                        .min_w(px(0.0))
                        .child(
                            text(move || folder.get().as_ref().map_or_else(
                                || "No sync folder".to_string(),
                                |path| format!("Folder connected · {}", crate::folder_sync::display_name(path)),
                            ))
                            .text_size(px(13.0))
                            .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || {
                                let last = last_sync.get();
                                if last.is_empty() {
                                    crate::folder_sync::BACKUP_FILE.to_string()
                                } else {
                                    format!("Last sync {last}")
                                }
                            })
                            .text_size(px(10.0))
                            .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(ui::action_button(
                    if folder.get().is_some() { "Change folder" } else { "Choose folder" },
                    false,
                    choose,
                ))
                .child(ui::action_button("Sync now", false, sync_now))
                .child(
                    when(move || folder.get().is_some(), move || {
                        ui::action_button("Disconnect", true, disconnect).into_element()
                    })
                    .otherwise(|| view().h(px(0.0)).into_element()),
                ),
        )
        .child(
            text(move || status.get())
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn data_card(
    server: String,
    token: String,
    status: Signal<String>,
    admin: bool,
) -> impl IntoElement {
    let pending = signal(None::<serde_json::Value>);
    let file_name = if admin {
        "recurrency-admin-backup.json"
    } else {
        "recurrency-backup.json"
    };
    let file_detail = if admin {
        "Every table, including users, accounts and server settings"
    } else {
        "Accounts, subscriptions, transactions and debts"
    };
    let note = if admin {
        "Keep it private; it contains every server table."
    } else {
        "Keep it private; it contains financial data."
    };
    let warning = if admin {
        "All current users, financial data and server settings will be replaced."
    } else {
        "This replaces your current accounts, categories, subscriptions, transactions and debts."
    };
    let export = {
        let server = server.clone();
        let token = token.clone();
        move || {
            status.set("Preparing backup…".to_string());
            let server_value = server.clone();
            let token_value = token.clone();
            spawn(async move {
                let exported = if admin {
                    crate::api::admin_export_backup(server_value, token_value).await
                } else {
                    crate::api::export_backup(server_value, token_value).await
                };
                match exported {
                    Ok(json) => save_backup(file_name, &json, status).await,
                    Err(error) => status.set(error),
                }
            });
        }
    };
    let choose_restore = move || {
        status.set("Choose a backup…".to_string());
        spawn(async move {
            match pick_backup_text().await {
                Ok(Some(text)) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(value) => {
                        pending.set(Some(value));
                        status
                            .set("Backup loaded. Confirm to replace the current data.".to_string());
                    }
                    Err(_) => status.set("That file is not a Recurrency backup.".to_string()),
                },
                Ok(None) => status.set(String::new()),
                Err(error) => status.set(error),
            }
        });
    };
    let confirm_restore = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let Some(value) = pending.get() else {
                return;
            };
            let server = server.clone();
            let token = token.clone();
            status.set("Restoring backup…".to_string());
            spawn(async move {
                let restored = if admin {
                    crate::api::admin_import_backup(server, token, &value).await
                } else {
                    crate::api::import_backup(server, token, &value).await
                };
                match restored {
                    Ok(_) => {
                        pending.set(None);
                        status.set(
                            "Backup restored. Revisit each screen to see the new data.".to_string(),
                        );
                    }
                    Err(error) => status.set(format!("Could not restore the backup: {error}")),
                }
            });
        }
    };
    let save_card = ui::surface()
        .flex_grow(1.0)
        .basis(px(0.0))
        .p(px(20.0))
        .h(px(282.0))
        .gap(px(16.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .gap(px(12.0))
                .child(
                    view()
                        .h(px(40.0))
                        .w(px(40.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || rgb(0x10b981).opacity(0.12))
                        .child(
                            image(asset!("icons/download.svg"))
                                .size(px(20.0))
                                .tint_color(move || rgb(0x34d399)),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text("Save a copy")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Download everything as one file.")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .h(px(60.0))
                .p(px(12.0))
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .bg(move || theme().card)
                        .child(
                            image(asset!("icons/notes.svg"))
                                .size(px(18.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(file_name)
                                .text_size(px(14.0))
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(file_detail)
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_grow(1.0)
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    text(note)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .h(px(38.0))
                        .px(px(14.0))
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .rounded(px(6.0))
                        .bg(move || theme().primary)
                        .on_press(export)
                        .child(
                            image(asset!("icons/download.svg"))
                                .size(px(14.0))
                                .tint_color(move || theme().primary_foreground),
                        )
                        .child(
                            text("Download backup")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().primary_foreground),
                        ),
                ),
        )
        .into_element();
    let restore_card = ui::surface()
        .flex_grow(1.0)
        .basis(px(0.0))
        .p(px(20.0))
        .h(px(282.0))
        .gap(px(16.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .gap(px(12.0))
                .child(
                    view()
                        .h(px(40.0))
                        .w(px(40.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().primary.opacity(0.14))
                        .child(
                            image(asset!("icons/upload.svg"))
                                .size(px(20.0))
                                .tint_color(move || theme().primary),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text("Bring it back")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Restore from a previous backup file.")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .min_h(px(180.0))
                .flex_grow(1.0)
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(999.0))
                        .bg(move || theme().background)
                        .child(
                            image(asset!("icons/upload.svg"))
                                .size(px(17.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    text("Drop a backup here")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("or choose a .json file")
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .h(px(36.0))
                        .px(px(12.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().card)
                        .accessibility_label("Choose backup file")
                        .on_press(choose_restore)
                        .child(
                            text("Choose file")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
        .child(
            when(
                move || pending.get().is_some(),
                move || {
                    view()
                        .w_full()
                        .mt(px(16.0))
                        .p(px(16.0))
                        .gap(px(8.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().destructive.opacity(0.4))
                        .bg(move || theme().destructive.opacity(0.08))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    image(asset!("icons/shield.svg"))
                                        .size(px(16.0))
                                        .tint_color(move || theme().destructive),
                                )
                                .child(
                                    text("Restore this backup?")
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().destructive),
                                ),
                        )
                        .child(
                            text(warning)
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            view()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(8.0))
                                .mt(px(4.0))
                                .child(crate::components::button::button(
                                    crate::components::button::ButtonProps::new("Restore backup")
                                        .variant(
                                            crate::components::button::ButtonVariant::Destructive,
                                        )
                                        .size(crate::components::button::ButtonSize::Sm)
                                        .on_press(confirm_restore.clone()),
                                ))
                                .child(crate::components::button::button(
                                    crate::components::button::ButtonProps::new("Cancel")
                                        .variant(crate::components::button::ButtonVariant::Outline)
                                        .size(crate::components::button::ButtonSize::Sm)
                                        .on_press(move || {
                                            pending.set(None);
                                            status.set(String::new());
                                        }),
                                )),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element();
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            ui::surface()
                .p(px(0.0))
                .child(
                    view()
                        .w_full()
                        .px(px(24.0))
                        .pt(px(18.0))
                        .pb(px(24.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .flex_row()
                        .items_start()
                        .justify_between()
                        .gap(px(12.0))
                        .child(
                            view()
                                .flex_row()
                                .items_start()
                                .gap(px(16.0))
                                .child(
                                    view()
                                        .h(px(44.0))
                                        .w(px(44.0))
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(12.0))
                                        .bg(move || theme().primary.opacity(0.10))
                                        .child(
                                            image(asset!("icons/shield.svg"))
                                                .size(px(22.0))
                                                .tint_color(move || theme().primary),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_col()
                                        .child(
                                            text("Backup & restore")
                                                .text_size(px(18.0))
                                                .line_height(px(28.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(move || theme().foreground),
                                        )
                                        .child(
                                            text("Keep a portable copy of your data, or bring it back in one step.")
                                                .text_size(px(14.0))
                                                .line_height(px(20.0))
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                ),
                        )
                        .child(
                            text("JSON")
                                .text_size(px(11.0))
                                .px(px(8.0))
                                .py(px(3.0))
                                .rounded(px(999.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .flex_row()
                        .gap(px(16.0))
                        .child(save_card)
                        .child(restore_card),
                ),
        )
        .child(
            text(move || status.get())
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn server_card(server: String, _session: Signal<Session>) -> impl IntoElement {
    crate::components::web_server_qr::web_server_qr(server)
}

const LATEST_APK_URL: &str =
    "https://github.com/badwatt/recurrency/releases/latest/download/recurrency-aarch64.apk";

const REPO_URL: &str = "https://github.com/badwatt/recurrency";

fn about_card() -> impl IntoElement {
    let release = resource(
        || Some(()),
        |(), _| async { Ok::<_, String>(crate::api::latest_release().await) },
    );
    let show_notes = signal(false);
    let notes = move || {
        release
            .get()
            .flatten()
            .map(|release| release.body)
            .filter(|body| !body.trim().is_empty())
            .unwrap_or_else(|| "Release notes are available on GitHub.".to_string())
    };
    let release_url = move || {
        release
            .get()
            .flatten()
            .map(|release| release.html_url)
            .unwrap_or_else(|| format!("{REPO_URL}/releases"))
    };
    let version = move || {
        release
            .get()
            .flatten()
            .map(|release| release.tag_name.trim_start_matches('v').to_string())
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
    };
    let hero = ui::surface()
        .p(px(32.0))
        .rounded(px(16.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(24.0))
                .child(
                    view()
                        .flex_grow(1.0)
                        .basis(px(0.0))
                        .min_w(px(0.0))
                        .flex_col()
                        .gap(px(5.0))
                        .child(
                            text("RECURRENCY")
                                .text_size(px(12.0))
                                .letter_spacing(px(2.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text("Every expense is a subscription.")
                                .text_size(px(30.0))
                                .line_height(px(36.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("A small, self-hosted tool for seeing what you keep paying for — and what you would subscribe to again today.")
                                .max_w(px(576.0))
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .px(px(20.0))
                        .py(px(20.0))
                        .gap(px(2.0))
                        .flex_shrink_0()
                        .rounded(px(12.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .bg(move || theme().background)
                        .child(
                            text("INSTALLED VERSION")
                                .text_size(px(11.0))
                                .letter_spacing(px(1.6))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .text_size(px(24.0))
                                .line_height(px(32.0))
                                .font_family(crate::theme::MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
        .into_element();
    let whats_new = ui::surface()
        .flex_grow(1.0)
        .basis(px(0.0))
        .p(px(24.0))
        .gap(px(10.0))
        .child(
            view()
                .flex_col()
                .child(
                    text("What's new")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(move || format!("Release notes for v{}", version()))
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(view().mt(px(9.0)).child(ui::action_button(
            if show_notes.get() {
                "Hide notes"
            } else {
                "Show changelog"
            },
            false,
            move || show_notes.set(!show_notes.get()),
        )))
        .child(
            when(
                move || show_notes.get(),
                move || {
                    view()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            scroll().max_h(px(240.0)).child(
                                text(notes())
                                    .text_size(px(11.0))
                                    .line_height(px(17.0))
                                    .text_color(move || theme().muted_foreground),
                            ),
                        )
                        .child(ui::action_button(
                            "Open release on GitHub",
                            false,
                            move || {
                                let url = release_url();
                                spawn(async move {
                                    let _ = open_url(url).await;
                                });
                            },
                        ))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element();
    let enjoying = ui::surface()
        .flex_grow(1.0)
        .basis(px(0.0))
        .p(px(24.0))
        .gap(px(10.0))
        .child(
            text("Enjoying Recurrency?")
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text("A star helps other people find a quieter way to track their subscriptions.")
                .text_size(px(14.0))
                .line_height(px(20.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            button()
                .cursor_pointer()
                .self_start()
                .h(px(36.0))
                .px(px(12.0))
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .rounded(px(6.0))
                .bg(move || theme().primary)
                .on_press(move || {
                    let url = REPO_URL.to_string();
                    spawn(async move {
                        let _ = open_url(url).await;
                    });
                })
                .child(
                    text("★")
                        .text_size(px(14.0))
                        .text_color(move || theme().primary_foreground),
                )
                .child(
                    text("Star on GitHub")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(16.0))
                .mt(px(13.0))
                .pt(px(10.0))
                .border_t(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .child(about_link("Report an issue", format!("{REPO_URL}/issues")))
                .child(about_link("Documentation", format!("{REPO_URL}#readme"))),
        )
        .into_element();
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(hero)
        .child(
            view()
                .w_full()
                .flex_row()
                .items_stretch()
                .gap(px(16.0))
                .child(whats_new)
                .child(enjoying),
        )
        .when(!cfg!(target_os = "android"), |view| {
            view.child(crate::components::web_server_qr::qr_card(
                "Android app",
                "Scan the code with an Android device to download the latest APK.",
                LATEST_APK_URL.to_string(),
                "recurrency-aarch64.apk".to_string(),
            ))
        })
}

fn about_link(label: &'static str, url: String) -> impl IntoElement {
    button()
        .cursor_pointer()
        .on_press(move || {
            let url = url.clone();
            spawn(async move {
                let _ = open_url(url).await;
            });
        })
        .child(
            text(label)
                .text_size(px(12.0))
                .text_color(move || theme().muted_foreground),
        )
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
    clear_api_key: bool,
}

fn summary_form(
    frequency: Signal<String>,
    weekday: Signal<String>,
    day: Signal<String>,
    period: Signal<String>,
    time: Signal<String>,
    timezone: Signal<String>,
    second: Option<&'static str>,
) -> AnyElement {
    let mut items = vec![
        string_select(StringSelectProps::new(
            "Frequency",
            frequency,
            vec![
                ("daily".to_string(), "Daily".to_string()),
                ("weekly".to_string(), "Weekly".to_string()),
                ("monthly".to_string(), "Monthly".to_string()),
            ],
        ))
        .into_element(),
    ];
    match second {
        Some("weekday") => items.push(
            string_select(StringSelectProps::new(
                "Weekday",
                weekday,
                vec![
                    ("1".to_string(), "Monday".to_string()),
                    ("2".to_string(), "Tuesday".to_string()),
                    ("3".to_string(), "Wednesday".to_string()),
                    ("4".to_string(), "Thursday".to_string()),
                    ("5".to_string(), "Friday".to_string()),
                    ("6".to_string(), "Saturday".to_string()),
                    ("7".to_string(), "Sunday".to_string()),
                ],
            ))
            .into_element(),
        ),
        Some("day") => {
            let mut days = (1..=31)
                .map(|value| (value.to_string(), value.to_string()))
                .collect::<Vec<_>>();
            days.push(("0".to_string(), "Last day".to_string()));
            items.push(
                string_select(StringSelectProps::new("Day of month", day, days)).into_element(),
            );
        }
        _ => {}
    }
    items.push(
        string_select(StringSelectProps::new(
            "Report period",
            period,
            vec![
                ("current".to_string(), "Current month".to_string()),
                ("next".to_string(), "Next month".to_string()),
            ],
        ))
        .into_element(),
    );
    items.push(ui::field("Time", time, "09:00").into_element());
    items.push(crate::components::timezone_select::timezone_select(timezone).into_element());
    view()
        .w_full()
        .grid()
        .grid_template_columns(SharedString::from("repeat(2, minmax(0, 1fr))"))
        .gap(px(12.0))
        .children(items)
        .into_element()
}

fn notifications_card(server: String, token: String) -> impl IntoElement {
    let summary_data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_summary(server, token).await }
        }
    });
    let summary_enabled = signal(false);
    let summary_frequency = signal("daily".to_string());
    let summary_time = signal("09:00".to_string());
    let summary_timezone = signal("UTC".to_string());
    let summary_weekday = signal("1".to_string());
    let summary_day = signal("1".to_string());
    let summary_period = signal("current".to_string());
    let summary_next = signal(String::new());
    let summary_error = signal(String::new());
    let summary_status = signal(String::new());
    let summary_loaded = signal(false);
    effect(move || {
        if summary_loaded.get() {
            return;
        }
        if let Some(summary) = summary_data.get() {
            summary_enabled.set(summary.enabled);
            summary_frequency.set(if summary.frequency.is_empty() {
                "daily".to_string()
            } else {
                summary.frequency
            });
            summary_time.set(format!("{:02}:{:02}", summary.hour, summary.minute));
            summary_timezone.set(if summary.timezone.is_empty() {
                "UTC".to_string()
            } else {
                summary.timezone
            });
            summary_weekday.set(summary.weekday.unwrap_or(1).to_string());
            summary_day.set(summary.day_of_month.unwrap_or(1).to_string());
            summary_period.set(if summary.period.is_empty() {
                "current".to_string()
            } else {
                summary.period
            });
            summary_next.set(summary.next_run_at.unwrap_or_default());
            summary_error.set(summary.last_error.unwrap_or_default());
            summary_loaded.set(true);
        }
    });
    let save_summary = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let time = summary_time.get();
            let mut parts = time.split(':');
            let hour = parts
                .next()
                .and_then(|value| value.parse::<i16>().ok())
                .unwrap_or(9);
            let minute = parts
                .next()
                .and_then(|value| value.parse::<i16>().ok())
                .unwrap_or(0);
            let frequency = summary_frequency.get();
            let weekday =
                (frequency == "weekly").then(|| summary_weekday.get().parse::<i16>().unwrap_or(1));
            let day =
                (frequency == "monthly").then(|| summary_day.get().parse::<i16>().unwrap_or(1));
            let server = server.clone();
            let token = token.clone();
            let timezone = summary_timezone.get();
            let period = summary_period.get();
            let enabled = summary_enabled.get();
            spawn(async move {
                match crate::api::save_summary(
                    server, token, enabled, &frequency, weekday, day, hour, minute, &timezone,
                    &period,
                )
                .await
                {
                    Ok(_) => {
                        summary_status.set("Summary schedule saved.".to_string());
                        summary_data.refetch();
                    }
                    Err(message) => summary_status.set(message),
                }
            });
        }
    };
    let send_summary = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::send_summary_now(server, token).await {
                    Ok(()) => summary_status.set("Summary sent.".to_string()),
                    Err(message) => summary_status.set(message),
                }
            });
        }
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(crate::delivery::delivery_card(
            server.clone(),
            token.clone(),
        ))
        .child(
            ui::surface()
                .p(px(0.0))
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .flex_row()
                        .items_start()
                        .justify_between()
                        .gap(px(12.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .child(
                            view()
                                .flex_col()
                                .child(
                                    text("Dashboard summary")
                                        .text_size(px(18.0))
                                        .line_height(px(28.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text("Sent to every active delivery channel")
                                        .text_size(px(14.0))
                                        .line_height(px(20.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(shared_toggle(
                            summary_enabled.get(),
                            "Dashboard summary".to_string(),
                            move |value| summary_enabled.set(value),
                        )),
                )
                .child(
                    view().w_full().p(px(24.0)).child(
                        when(
                            move || summary_frequency.get() == "monthly",
                            move || {
                                summary_form(
                                    summary_frequency,
                                    summary_weekday,
                                    summary_day,
                                    summary_period,
                                    summary_time,
                                    summary_timezone,
                                    Some("day"),
                                )
                            },
                        )
                        .otherwise(move || {
                            when(
                                move || summary_frequency.get() == "weekly",
                                move || {
                                    summary_form(
                                        summary_frequency,
                                        summary_weekday,
                                        summary_day,
                                        summary_period,
                                        summary_time,
                                        summary_timezone,
                                        Some("weekday"),
                                    )
                                },
                            )
                            .otherwise(move || {
                                summary_form(
                                    summary_frequency,
                                    summary_weekday,
                                    summary_day,
                                    summary_period,
                                    summary_time,
                                    summary_timezone,
                                    None,
                                )
                            })
                            .into_element()
                        }),
                    ),
                )
                .child(
                    view()
                        .w_full()
                        .px(px(24.0))
                        .pb(px(24.0))
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .justify_between()
                        .gap(px(12.0))
                        .child(
                            view()
                                .flex_col()
                                .child(
                                    text(move || {
                                        let next = summary_next.get();
                                        if next.is_empty() {
                                            "No run scheduled".to_string()
                                        } else {
                                            format!("Next run: {next}")
                                        }
                                    })
                                    .text_size(px(12.0))
                                    .text_color(move || theme().muted_foreground),
                                )
                                .when(!summary_error.get().is_empty(), |col| {
                                    col.child(
                                        text(move || {
                                            format!("Last attempt failed: {}", summary_error.get())
                                        })
                                        .text_size(px(12.0))
                                        .text_color(move || theme().destructive),
                                    )
                                }),
                        )
                        .child(
                            view()
                                .flex_row()
                                .flex_wrap()
                                .items_center()
                                .gap(px(8.0))
                                .child(crate::components::button::button(
                                    crate::components::button::ButtonProps::new("Send now")
                                        .variant(crate::components::button::ButtonVariant::Outline)
                                        .size(crate::components::button::ButtonSize::Md)
                                        .on_press(send_summary),
                                ))
                                .child(crate::components::button::button(
                                    crate::components::button::ButtonProps::new("Save schedule")
                                        .size(crate::components::button::ButtonSize::Md)
                                        .on_press(save_summary),
                                )),
                        ),
                ),
        )
}

fn ai_hero(configured: impl Fn() -> bool + 'static) -> AnyElement {
    ui::surface()
        .p(px(32.0))
        .pb(px(40.0))
        .rounded(px(16.0))
        .gap(px(16.0))
        .bg(move || theme().primary.opacity(0.06))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(16.0))
                .child(
                    view()
                        .h(px(64.0))
                        .w(px(64.0))
                        .mt(px(-16.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(16.0))
                        .bg(move || theme().primary.opacity(0.16))
                        .child(
                            image(asset!("icons/sparkles.svg"))
                                .size(px(28.0))
                                .tint_color(move || theme().primary),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(3.0))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    text("AI assistant")
                                        .text_size(px(24.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(badge(BadgeProps::new("BYOK").tone(BadgeTone::Outline))),
                        )
                        .child(
                            text("Connect an OpenAI-compatible model to create and change your subscriptions, transactions and debts. Choose your provider, model and how you want to dictate.")
                                .max_w(px(680.0))
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .mt(px(4.0))
                .gap(px(8.0))
                .child(ai_chip(move || {
                    if configured() {
                        "Key configured".to_string()
                    } else {
                        "No key configured".to_string()
                    }
                }))
                .child(ai_chip(|| "Automatic".to_string()))
                .child(ai_chip(|| "Key encrypted at rest".to_string())),
        )
        .into_element()
}

fn ai_chip(label: impl Fn() -> String + 'static) -> impl IntoElement {
    view()
        .h(px(24.0))
        .px(px(10.0))
        .items_center()
        .rounded(px(999.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .child(
            text(label)
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn ai_card(server: String, token: String) -> impl IntoElement {
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_ai_settings(server, token).await }
        }
    });
    let provider = signal("openai".to_string());
    let base_url = signal("https://api.openai.com/v1".to_string());
    let model = signal("gpt-4o-mini".to_string());
    let api_key = signal(String::new());
    let clear_key = signal(false);
    let models = signal(Vec::<String>::new());
    let models_loading = signal(false);
    let models_error = signal(String::new());
    let status = signal(String::new());
    let loaded = signal(false);
    effect(move || {
        if loaded.get() {
            return;
        }
        if let Some(settings) = data.get() {
            if !settings.provider.trim().is_empty() {
                provider.set(settings.provider);
            }
            if !settings.base_url.trim().is_empty() {
                base_url.set(settings.base_url);
            }
            if !settings.model.trim().is_empty() {
                model.set(settings.model);
            }
            loaded.set(true);
        }
    });
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
                clear_api_key: clear_key.get(),
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
                clear_api_key: clear_key.get(),
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
    let refresh_models = {
        let server = server.clone();
        let token = token.clone();
        move || {
            models_loading.set(true);
            models_error.set(String::new());
            let server = server.clone();
            let token = token.clone();
            let base = base_url.get().trim().to_string();
            spawn(async move {
                match crate::api::ai_models(server, token, (!base.is_empty()).then_some(base)).await
                {
                    Ok(values) => models.set(values),
                    Err(message) => models_error.set(message),
                }
                models_loading.set(false);
            });
        }
    };
    let configured = move || data.get().map_or(false, |settings| settings.configured);
    view()
        .w_full()
        .flex_col()
        .gap(px(20.0))
        .child(ai_hero(configured))
        .child(
            ui::surface()
                .p(px(0.0))
                .min_h(px(528.0))
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(16.0))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(12.0))
                                .child(
                                    view()
                                        .h(px(48.0))
                                        .w(px(48.0))
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(12.0))
                                        .bg(move || theme().primary.opacity(0.14))
                                        .child(
                                            image(asset!("icons/sparkles.svg"))
                                                .size(px(22.0))
                                                .tint_color(move || theme().primary),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_col()
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    text("Model connection")
                                                        .text_size(px(18.0))
                                                        .line_height(px(28.0))
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(move || theme().foreground),
                                                )
                                                .child(badge(BadgeProps::new("Not configured").tone(BadgeTone::Outline))),
                                        )
                                        .child(
                                            text("Bring your own key. Keys stay encrypted on the server and are never returned to this device.")
                                                .text_size(px(14.0))
                                                .line_height(px(20.0))
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                ),
                        )
                        .child(shared_toggle(
                            data.get().map_or(false, |settings| settings.enabled),
                            "Enabled".to_string(),
                            move |_| save(),
                        )),
                )
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .gap(px(12.0))
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .gap(px(12.0))
                                .child(
                                    view()
                                        .flex_grow(1.0)
                                        .basis(px(0.0))
                                        .min_w(px(0.0))
                                        .gap(px(4.0))
                                        .child(string_select(StringSelectProps::new(
                                            "Provider",
                                            provider,
                                            vec![
                                                ("openai".to_string(), "OpenAI".to_string()),
                                                ("groq".to_string(), "Groq".to_string()),
                                                ("openrouter".to_string(), "OpenRouter".to_string()),
                                                ("ollama".to_string(), "Ollama".to_string()),
                                                ("lmstudio".to_string(), "LM Studio".to_string()),
                                                ("custom".to_string(), "Custom".to_string()),
                                            ],
                                        )))
                                        .child(
                                            text("GPT models with your own key.")
                                                .text_size(px(11.0))
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_grow(1.0)
                                        .basis(px(0.0))
                                        .min_w(px(0.0))
                                        .gap(px(4.0))
                                        .child(ui::field("API base URL", base_url, "https://api.openai.com/v1"))
                                        .child(
                                            text("OpenAI")
                                                .text_size(px(11.0))
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                ),
                        )
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .items_end()
                                .gap(px(8.0))
                                .child(
                                    view()
                                        .flex_grow(1.0)
                                        .basis(px(0.0))
                                        .min_w(px(0.0))
                                        .child(ui::field("Model", model, "gpt-4o-mini")),
                                )
                                .child(ui::action_button(
                                    if models_loading.get() { "Loading models…" } else { "Refresh models" },
                                    false,
                                    refresh_models,
                                )),
                        )
                        .child(ui::field("API key", api_key, "Paste your provider API key"))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    image(asset!("icons/shield.svg"))
                                        .size(px(13.0))
                                        .tint_color(move || theme().muted_foreground),
                                )
                                .child(
                                    text("Encrypted at rest. Never sent back to the client.")
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(view().w_full().flex_grow(1.0))
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .gap(px(8.0))
                                .pt(px(16.0))
                                .border_t(px(1.0))
                                .border_color(move || theme().border.opacity(0.5))
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            image(asset!("icons/sparkles.svg"))
                                                .size(px(13.0))
                                                .tint_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            text(move || {
                                                if configured() {
                                                    "Key stored".to_string()
                                                } else {
                                                    "Key not configured".to_string()
                                                }
                                            })
                                            .text_size(px(12.0))
                                            .text_color(move || theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    button()
                                        .cursor_pointer()
                                        .h(px(40.0))
                                        .px(px(14.0))
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .rounded(px(6.0))
                                        .bg(move || theme().primary)
                                        .on_press(move || configure())
                                        .child(
                                            image(asset!("icons/lock.svg"))
                                                .size(px(14.0))
                                                .tint_color(move || theme().primary_foreground),
                                        )
                                        .child(
                                            text("Save AI settings")
                                                .text_size(px(14.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().primary_foreground),
                                        ),
                                ),
                        ),
                ),
        )
        .child(voice_card(server.clone(), token.clone()))
        .child(
            view()
                .w_full()
                .p(px(16.0))
                .flex_row()
                .items_start()
                .gap(px(12.0))
                .rounded(px(12.0))
                .border(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .bg(move || theme().muted.opacity(0.2))
                .child(
                    image(asset!("icons/sparkles.svg"))
                        .size(px(14.0))
                        .tint_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .child(
                            text("Works with your data")
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("The assistant receives a compact snapshot of your subscriptions, transactions, debts, accounts and categories. Actions run with your permissions.")
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
}

fn voice_card(server: String, token: String) -> impl IntoElement {
    let mode = signal(crate::voice::saved());
    let ai = resource(
        || Some(()),
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_ai_settings(server, token).await }
        },
    );
    let configured = move || ai.get().is_some_and(|settings| settings.configured);
    let icon = |option: crate::voice::VoiceMode| -> ImageAsset {
        match option {
            crate::voice::VoiceMode::Auto => asset!("icons/sparkles.svg"),
            crate::voice::VoiceMode::Native => asset!("icons/smartphone.svg"),
            crate::voice::VoiceMode::Browser => asset!("icons/globe.svg"),
            crate::voice::VoiceMode::Provider => asset!("icons/cloud.svg"),
            crate::voice::VoiceMode::Local => asset!("icons/cpu.svg"),
        }
    };
    ui::surface()
        .p(px(0.0))
        .rounded(px(16.0))
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .flex_row()
                .flex_wrap()
                .items_start()
                .gap(px(16.0))
                .border_b(px(1.0))
                .border_color(move || theme().border.opacity(0.6))
                .child(
                    view()
                        .h(px(48.0))
                        .w(px(48.0))
                        .flex_shrink_0()
                        .items_center()
                        .justify_center()
                        .rounded(px(16.0))
                        .bg(move || rgb(0x0ea5e9).opacity(0.10))
                        .child(
                            image(asset!("icons/mic.svg"))
                                .size(px(24.0))
                                .tint_color(move || rgb(0x38bdf8)),
                        ),
                )
                .child(
                    view()
                        .flex_grow(1.0)
                        .basis(px(0.0))
                        .min_w(px(0.0))
                        .flex_col()
                        .child(
                            text("Voice input")
                                .text_size(px(18.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Dictate into the AI chat. Choose the engine that fits this device.")
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .h(px(24.0))
                        .px(px(10.0))
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .rounded(px(999.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .child(
                            image(asset!("icons/shield.svg"))
                                .size(px(11.0))
                                .tint_color(move || theme().muted_foreground),
                        )
                        .child(
                            text("Private by choice")
                                .text_size(px(10.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .flex_col()
                .gap(px(20.0))
                .child(
                    view()
                        .w_full()
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(2, minmax(0, 1fr))"))
                        .gap(px(8.0))
                        .children(crate::voice::VoiceMode::ALL.into_iter().map(move |option| {
                            let active = mode.get() == option;
                            let provider_missing = move || {
                                option == crate::voice::VoiceMode::Provider && !configured()
                            };
                            let available = move || {
                                option.available() && !provider_missing()
                            };
                            let hint = move || {
                                if provider_missing() {
                                    "Configure a model first".to_string()
                                } else {
                                    option.hint().to_string()
                                }
                            };
                            view()
                                .w_full()
                                .cursor_pointer()
                                .p(px(14.0))
                                .flex_row()
                                .items_start()
                                .gap(px(12.0))
                                .rounded(px(12.0))
                                .border(px(1.0))
                                .border_color(move || {
                                    if active {
                                        theme().primary.opacity(0.45)
                                    } else {
                                        theme().border.opacity(0.6)
                                    }
                                })
                                .bg(move || {
                                    if active {
                                        theme().primary.opacity(0.05)
                                    } else {
                                        theme().background.opacity(0.35)
                                    }
                                })
                                .opacity(move || if available() { 1.0_f32 } else { 0.55_f32 })
                                .on_press(move || {
                                    if available() {
                                        mode.set(option);
                                        crate::voice::save(option);
                                    }
                                })
                                .child(
                                    view()
                                        .h(px(32.0))
                                        .w(px(32.0))
                                        .flex_shrink_0()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(8.0))
                                        .bg(move || {
                                            if active {
                                                theme().primary.opacity(0.12)
                                            } else {
                                                theme().muted
                                            }
                                        })
                                        .child(
                                            image(icon(option))
                                                .size(px(18.0))
                                                .tint_color(move || {
                                                    if active {
                                                        theme().primary
                                                    } else {
                                                        theme().muted_foreground
                                                    }
                                                }),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_col()
                                        .min_w(px(0.0))
                                        .child(
                                            view()
                                                .flex_row()
                                                .flex_wrap()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    text(option.label())
                                                        .text_size(px(14.0))
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(move || theme().foreground),
                                                )
                                                .when(option == crate::voice::VoiceMode::Auto, |row| {
                                                    row.child(
                                                        view()
                                                            .px(px(6.0))
                                                            .py(px(2.0))
                                                            .rounded(px(999.0))
                                                            .bg(move || crate::theme::success().opacity(0.12))
                                                            .child(
                                                                text("RECOMMENDED")
                                                                    .text_size(px(9.0))
                                                                    .font_weight(FontWeight::SEMIBOLD)
                                                                    .letter_spacing(px(0.5))
                                                                    .text_color(move || crate::theme::success()),
                                                            ),
                                                    )
                                                }),
                                        )
                                        .child(
                                            text(option.description())
                                                .text_size(px(12.0))
                                                .line_height(px(16.0))
                                                .text_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            text(hint)
                                                .text_size(px(10.0))
                                                .text_color(move || rgb(0xd97706)),
                                        ),
                                )
                                .into_element()
                        })),
                )
                .child(
                    view()
                        .w_full()
                        .p(px(12.0))
                        .flex_row()
                        .items_start()
                        .gap(px(8.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.4))
                        .bg(move || theme().muted.opacity(0.2))
                        .child(
                            image(asset!("icons/cpu.svg"))
                                .size(px(13.0))
                                .tint_color(move || theme().muted_foreground),
                        )
                        .child(
                            text("Provider transcription uses the same encrypted API key as chat. Browser and Android speech may use services provided by your operating system or browser.")
                                .text_size(px(11.0))
                                .line_height(px(16.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .into_element()
}

fn exchange_rates_card(server: String, token: String) -> impl IntoElement {
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_currency_settings(server, token).await }
        }
    });
    let provider = signal("frankfurter".to_string());
    let key = signal(String::new());
    let configured = signal(false);
    let status = signal(String::new());
    let loaded = signal(false);
    effect(move || {
        if loaded.get() {
            return;
        }
        if let Some(settings) = data.get() {
            provider.set(if settings.provider.is_empty() {
                "frankfurter".to_string()
            } else {
                settings.provider
            });
            configured.set(settings.fixer_configured);
            loaded.set(true);
        }
    });
    let save = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let provider = provider.get();
            let api_key = (!key.get().trim().is_empty()).then(|| key.get());
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::save_currency_settings(server, token, &provider, api_key, false)
                    .await
                {
                    Ok(settings) => {
                        configured.set(settings.fixer_configured);
                        key.set(String::new());
                        crate::components::toast::success("Exchange-rate settings saved.");
                        status.set("Exchange-rate settings saved.".to_string());
                        data.refetch();
                    }
                    Err(message) => status.set(message),
                }
            });
        }
    };
    let clear_key = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::save_currency_settings(server, token, "frankfurter", None, true)
                    .await
                {
                    Ok(_) => {
                        provider.set("frankfurter".to_string());
                        configured.set(false);
                        data.refetch();
                        status.set("Fixer key removed.".to_string());
                    }
                    Err(message) => status.set(message),
                }
            });
        }
    };
    let clear_key = std::rc::Rc::new(clear_key);
    settings_section(
        "Exchange rates",
        "Choose where automatic currency conversion gets its daily rates.",
        view()
            .w_full()
            .flex_col()
            .gap(px(12.0))
            .child(string_select(StringSelectProps::new(
                "Provider",
                provider,
                vec![
                    (
                        "frankfurter".to_string(),
                        "Frankfurter — free, no API key".to_string(),
                    ),
                    (
                        "fixer".to_string(),
                        "Fixer — requires an API key".to_string(),
                    ),
                ],
            )))
            .child(
                view()
                    .w_full()
                    .flex_col()
                    .gap(move || {
                        if provider.get() == "fixer" {
                            px(12.0)
                        } else {
                            px(0.0)
                        }
                    })
                    .child(
                        when(
                            move || provider.get() == "fixer",
                            move || {
                                ui::field(
                                    if configured.get() {
                                        "Fixer API key (saved — enter a new key to replace it)"
                                    } else {
                                        "Fixer API key"
                                    },
                                    key,
                                    "Paste your Fixer API key",
                                )
                                .into_element()
                            },
                        )
                        .otherwise(|| view().h(px(0.0)).into_element()),
                    )
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                text("Rates are cached for six hours.")
                                    .text_size(px(12.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .flex_wrap()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(ui::action_button("Save rates", false, save))
                                    .child(
                                        when(
                                            move || configured.get(),
                                            move || {
                                                let clear_key = clear_key.clone();
                                                ui::action_button("Remove key", true, move || {
                                                    clear_key()
                                                })
                                                .into_element()
                                            },
                                        )
                                        .otherwise(|| view().h(px(0.0)).into_element()),
                                    ),
                            ),
                    ),
            )
            .child(
                when(
                    move || !status.get().is_empty(),
                    move || {
                        text(move || status.get())
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground)
                            .into_element()
                    },
                )
                .otherwise(|| view().h(px(0.0)).into_element()),
            )
            .into_element(),
    )
}

fn money_card(server: String, token: String, modal: ui::ModalHost) -> impl IntoElement {
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_money_counts(server, token).await }
        }
    });
    let accounts_tab = signal(true);
    let account_name = signal(String::new());
    let account_icon = signal("wallet".to_string());
    let account_color = signal(crate::components::color_picker::DEFAULT_COLOR.to_string());
    let account_edit = signal(None::<Uuid>);
    let account_error = signal(String::new());
    let account_busy = signal(false);
    let category_name = signal(String::new());
    let category_icon = signal("tag".to_string());
    let category_color = signal(crate::components::color_picker::DEFAULT_COLOR.to_string());
    let category_edit = signal(None::<Uuid>);
    let category_error = signal(String::new());
    let category_busy = signal(false);

    let remove_account: std::rc::Rc<dyn Fn(Uuid)> = std::rc::Rc::new({
        let server = server.clone();
        let token = token.clone();
        move |id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::delete_account_by_id(server, token, id).await {
                    Ok(()) => {
                        crate::components::toast::success("Account deleted.");
                        data.refetch();
                    }
                    Err(message) => crate::components::toast::error(message),
                }
            });
        }
    });
    let mark_default: std::rc::Rc<dyn Fn(Uuid)> = std::rc::Rc::new({
        let server = server.clone();
        let token = token.clone();
        move |id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::set_default_account(server, token, id).await {
                    Ok(_) => data.refetch(),
                    Err(message) => crate::components::toast::error(message),
                }
            });
        }
    });
    let remove_category: std::rc::Rc<dyn Fn(Uuid)> = std::rc::Rc::new({
        let server = server.clone();
        let token = token.clone();
        move |id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::delete_category_by_id(server, token, id).await {
                    Ok(()) => {
                        crate::components::toast::success("Category deleted.");
                        data.refetch();
                    }
                    Err(message) => crate::components::toast::error(message),
                }
            });
        }
    });
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
            let icon = (!account_icon.get().trim().is_empty()).then(|| account_icon.get());
            let color = (!account_color.get().trim().is_empty()).then(|| account_color.get());
            let server = server.clone();
            let token = token.clone();
            account_busy.set(true);
            spawn(async move {
                let result = match editing {
                    Some(id) => crate::api::update_account(
                        server,
                        token,
                        id,
                        &name,
                        icon.as_deref(),
                        color.as_deref(),
                    )
                    .await
                    .map(|_| ()),
                    None => crate::api::create_account(
                        server,
                        token,
                        &name,
                        icon.as_deref(),
                        color.as_deref(),
                    )
                    .await
                    .map(|_| ()),
                };
                match result {
                    Ok(()) => {
                        account_error.set(String::new());
                        account_busy.set(false);
                        modal.close();
                        crate::components::toast::success("Account saved.");
                        data.refetch();
                    }
                    Err(message) => {
                        account_error.set(message);
                        account_busy.set(false);
                    }
                }
            });
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
            let icon = (!category_icon.get().trim().is_empty()).then(|| category_icon.get());
            let color = (!category_color.get().trim().is_empty()).then(|| category_color.get());
            let server = server.clone();
            let token = token.clone();
            category_busy.set(true);
            spawn(async move {
                let result = match editing {
                    Some(id) => crate::api::update_category(
                        server,
                        token,
                        id,
                        &name,
                        icon.as_deref(),
                        color.as_deref(),
                    )
                    .await
                    .map(|_| ()),
                    None => crate::api::create_category(
                        server,
                        token,
                        &name,
                        icon.as_deref(),
                        color.as_deref(),
                    )
                    .await
                    .map(|_| ()),
                };
                match result {
                    Ok(()) => {
                        category_error.set(String::new());
                        category_busy.set(false);
                        modal.close();
                        crate::components::toast::success("Category saved.");
                        data.refetch();
                    }
                    Err(message) => {
                        category_error.set(message);
                        category_busy.set(false);
                    }
                }
            });
        }
    };

    let open_account = {
        let save = save_account.clone();
        move || {
            account_name.set(String::new());
            account_icon.set("wallet".to_string());
            account_color.set(crate::components::color_picker::DEFAULT_COLOR.to_string());
            account_edit.set(None);
            account_error.set(String::new());
            let save = save.clone();
            modal.open(move || {
                account_dialog(
                    "Add Account",
                    account_name,
                    account_icon,
                    account_color,
                    account_error,
                    account_busy,
                    save.clone(),
                    modal,
                )
            });
        }
    };
    let open_category = {
        let save = save_category.clone();
        move || {
            category_name.set(String::new());
            category_icon.set("tag".to_string());
            category_color.set(crate::components::color_picker::DEFAULT_COLOR.to_string());
            category_edit.set(None);
            category_error.set(String::new());
            let save = save.clone();
            modal.open(move || {
                category_dialog(
                    "Add Category",
                    category_name,
                    category_icon,
                    category_color,
                    category_error,
                    category_busy,
                    save.clone(),
                    modal,
                )
            });
        }
    };
    let open_edit_account: std::rc::Rc<dyn Fn(Uuid, String, String, String)> = std::rc::Rc::new({
        let save = save_account.clone();
        move |id: Uuid, name: String, icon: String, color: String| {
            account_name.set(name);
            account_icon.set(icon);
            account_color.set(color);
            account_edit.set(Some(id));
            account_error.set(String::new());
            let save = save.clone();
            modal.open(move || {
                account_dialog(
                    "Edit Account",
                    account_name,
                    account_icon,
                    account_color,
                    account_error,
                    account_busy,
                    save.clone(),
                    modal,
                )
            });
        }
    });
    let open_edit_category: std::rc::Rc<dyn Fn(Uuid, String, String, String)> = std::rc::Rc::new({
        let save = save_category.clone();
        move |id: Uuid, name: String, icon: String, color: String| {
            category_name.set(name);
            category_icon.set(icon);
            category_color.set(color);
            category_edit.set(Some(id));
            category_error.set(String::new());
            let save = save.clone();
            modal.open(move || {
                category_dialog(
                    "Edit Category",
                    category_name,
                    category_icon,
                    category_color,
                    category_error,
                    category_busy,
                    save.clone(),
                    modal,
                )
            });
        }
    });

    let build_accounts: std::rc::Rc<dyn Fn() -> AnyElement> = std::rc::Rc::new(move || {
        let open_edit = open_edit_account.clone();
        let remove = remove_account.clone();
        let mark_default = mark_default.clone();
        let data_for_accounts = data.clone();
        view()
            .w_full()
            .grid()
            .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
            .gap(px(16.0))
            .child(each(
                move || data_for_accounts.get().unwrap_or_default().0,
                |item| item.account.id,
                move |item, _| {
                    let item = item.get();
                    let account = item.account;
                    let id = account.id;
                    let name = account.name.clone();
                    let icon = account.icon.clone().unwrap_or_else(|| "wallet".to_string());
                    let color = account.color.clone().unwrap_or_else(|| {
                        crate::components::color_picker::DEFAULT_COLOR.to_string()
                    });
                    let is_default = account.is_default;
                    let edit_name = name.clone();
                    let edit_icon = icon.clone();
                    let edit_color = color.clone();
                    let open_edit = open_edit.clone();
                    view()
                        .w_full()
                        .rounded(px(12.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .bg(move || theme().card)
                        .p(px(16.0))
                        .min_h(px(140.0))
                        .gap(px(12.0))
                        .child(
                            view()
                                .w_full()
                                .min_h(px(44.0))
                                .flex_row()
                                .items_center()
                                .gap(px(12.0))
                                .child(crate::components::entity_icon::entity_icon(
                                    &icon,
                                    crate::components::color_picker::color_value(&color),
                                    40.0,
                                ))
                                .child(
                                    view()
                                        .flex_col()
                                        .min_w(px(0.0))
                                        .child(
                                            text(name)
                                                .text_size(px(14.0))
                                                .line_height(px(20.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().foreground),
                                        )
                                        .when(is_default, |column| {
                                            column.child(
                                                text("Default")
                                                    .text_size(px(12.0))
                                                    .text_color(move || theme().muted_foreground),
                                            )
                                        }),
                                ),
                        )
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .justify_end()
                                .pt(px(8.0))
                                .border_t(px(1.0))
                                .border_color(move || theme().border.opacity(0.4))
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(4.0))
                                        .when(!is_default, |actions| {
                                            let mark_default = mark_default.clone();
                                            actions.child(ui::row_icon_button_labeled(
                                                asset!("icons/star.svg"),
                                                "Set account as default",
                                                false,
                                                move || mark_default(id),
                                            ))
                                        })
                                        .child(ui::row_icon_button_labeled(
                                            asset!("icons/pencil.svg"),
                                            "Edit account",
                                            false,
                                            move || {
                                                open_edit(
                                                    id,
                                                    edit_name.clone(),
                                                    edit_icon.clone(),
                                                    edit_color.clone(),
                                                );
                                            },
                                        ))
                                        .when(!is_default, |actions| {
                                            let remove = remove.clone();
                                            actions.child(ui::row_icon_button_labeled(
                                                asset!("icons/trash.svg"),
                                                "Delete account",
                                                true,
                                                move || remove(id),
                                            ))
                                        }),
                                ),
                        )
                        .into_element()
                },
            ))
            .into_element()
    });
    let build_categories: std::rc::Rc<dyn Fn() -> AnyElement> = std::rc::Rc::new(move || {
        let open_edit = open_edit_category.clone();
        let remove = remove_category.clone();
        let data_for_categories = data.clone();
        view()
            .w_full()
            .grid()
            .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
            .gap(px(16.0))
            .child(each(
                move || data_for_categories.get().unwrap_or_default().1,
                |item| item.category.id,
                move |item, _| {
                    let item = item.get();
                    let category = item.category;
                    let id = category.id;
                    let name = category.name.clone();
                    let icon = category.icon.clone().unwrap_or_else(|| "tag".to_string());
                    let color = category.color.clone().unwrap_or_else(|| {
                        crate::components::color_picker::DEFAULT_COLOR.to_string()
                    });
                    let is_system = category.is_system;
                    let is_default = category.is_default;
                    let subscriptions = item.subscription_count;
                    let transactions = item.transaction_count;
                    let edit_name = name.clone();
                    let edit_icon = icon.clone();
                    let edit_color = color.clone();
                    let open_edit = open_edit.clone();
                    view()
                    .w_full()
                    .rounded(px(12.0))
                    .border(px(1.0))
                    .border_color(move || theme().border.opacity(0.6))
                    .bg(move || theme().card)
                    .p(px(16.0))
                    .min_h(px(140.0))
                    .gap(px(12.0))
                    .child(
                        view()
                            .w_full()
                            .min_h(px(44.0))
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .child(crate::components::entity_icon::entity_icon(
                                &icon,
                                crate::components::color_picker::color_value(&color),
                                40.0,
                            ))
                            .child(
                                view()
                                    .flex_col()
                                    .min_w(px(0.0))
                                    .child(
                                        text(name)
                                            .text_size(px(14.0))
                                            .line_height(px(20.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(move || theme().foreground),
                                    )
                                    .when(is_default, |column| {
                                        column.child(
                                            text("Default")
                                                .text_size(px(12.0))
                                                .text_color(move || theme().muted_foreground),
                                        )
                                    })
                                    .when(is_system, |column| {
                                        column.child(
                                            text("System")
                                                .text_size(px(10.0))
                                                .text_color(move || theme().muted_foreground),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .pt(px(8.0))
                            .border_t(px(1.0))
                            .border_color(move || theme().border.opacity(0.4))
                            .child(
                                text(format!(
                                    "{subscriptions} subscriptions · {transactions} transactions"
                                ))
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                            )
                            .when(!is_system, |actions| {
                                let remove = remove.clone();
                                actions.child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(4.0))
                                        .child(ui::mini_icon_button(
                                            asset!("icons/pencil.svg"),
                                            false,
                                            move || {
                                                open_edit(
                                                    id,
                                                    edit_name.clone(),
                                                    edit_icon.clone(),
                                                    edit_color.clone(),
                                                );
                                            },
                                        ))
                                        .child(ui::mini_icon_button(
                                            asset!("icons/trash.svg"),
                                            true,
                                            move || remove(id),
                                        )),
                                )
                            }),
                    )
                    .into_element()
                },
            ))
            .into_element()
    });

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .justify_between()
                .gap(px(24.0))
                .pb(px(2.0))
                .mb(px(-6.0))
                .child(
                    view()
                        .flex_col()
                        .mt(px(-10.0))
                        .pb(px(4.0))
                        .child(
                            text("Accounts & categories")
                                .h(px(48.0))
                                .mb(px(-2.0))
                                .text_size(px(48.0))
                                .line_height(px(48.0))
                                .letter_spacing(px(-2.4))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Accounts and categories that organize your money")
                                .mt(px(6.0))
                                .text_size(px(16.0))
                                .line_height(px(24.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .h(px(40.0))
                        .px(px(16.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .gap(px(6.0))
                        .rounded(px(6.0))
                        .bg(move || theme().primary)
                        .on_press(move || {
                            if accounts_tab.get() {
                                open_account();
                            } else {
                                open_category();
                            }
                        })
                        .child(
                            image(asset!("icons/plus.svg"))
                                .size(px(15.0))
                                .tint_color(move || theme().primary_foreground),
                        )
                        .child(
                            text(move || {
                                if accounts_tab.get() {
                                    "Add Account"
                                } else {
                                    "Add Category"
                                }
                            })
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().primary_foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(16.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .child(settings_tab("Accounts", true, accounts_tab, true))
                .child(settings_tab("Categories", false, accounts_tab, true)),
        )
        .child(
            when(move || accounts_tab.get(), {
                let build = build_accounts.clone();
                move || build()
            })
            .otherwise({
                let build = build_categories.clone();
                move || build()
            }),
        )
}

fn settings_tab(
    label: &'static str,
    id: bool,
    selected: Signal<bool>,
    _active: bool,
) -> impl IntoElement {
    let active = move || selected.get() == id;
    button()
        .cursor_pointer()
        .h(px(36.0))
        .px(px(12.0))
        .items_center()
        .justify_center()
        .border_b(px(2.0))
        .border_color(move || {
            if active() {
                theme().primary
            } else {
                theme().card.opacity(0.0)
            }
        })
        .on_press(move || selected.set(id))
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(if active() {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(move || {
                    if active() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

const ENTITY_ICONS: &[&str] = &[
    "wallet",
    "tag",
    "receipt",
    "card",
    "subscriptions",
    "transactions",
    "debts",
    "calendar",
    "reports",
    "settings",
    "database",
    "bell",
    "clock",
    "trending",
    "notes",
];

fn entity_icon_field(icon: Signal<String>, color: Signal<String>) -> AnyElement {
    let open = signal(false);
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text("Icon")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(ui::action_button(
                    if open.get() {
                        "Hide icons"
                    } else {
                        "Choose icon"
                    },
                    false,
                    move || open.set(!open.get()),
                )),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(crate::components::entity_icon::entity_icon(
                    &icon.get(),
                    crate::components::color_picker::color_value(&color.get()),
                    32.0,
                ))
                .child(
                    text(move || icon.get())
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(
                move || open.get(),
                move || {
                    view()
                        .w_full()
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(8, minmax(0, 1fr))"))
                        .gap(px(4.0))
                        .p(px(6.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().background)
                        .children(ENTITY_ICONS.iter().map(|name| {
                            let value = (*name).to_string();
                            let active_value = value.clone();
                            let active = move || icon.get() == active_value;
                            button()
                                .cursor_pointer()
                                .h(px(30.0))
                                .w_full()
                                .items_center()
                                .justify_center()
                                .rounded(px(5.0))
                                .border(px(1.0))
                                .border_color(move || {
                                    if active() {
                                        theme().primary
                                    } else {
                                        theme().border
                                    }
                                })
                                .on_press(move || icon.set(value.clone()))
                                .child(crate::components::entity_icon::entity_icon(
                                    name,
                                    crate::components::color_picker::color_value(&color.get()),
                                    16.0,
                                ))
                                .into_element()
                        }))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element()
}

fn account_dialog(
    title: &'static str,
    name: Signal<String>,
    icon: Signal<String>,
    color: Signal<String>,
    error: Signal<String>,
    busy: Signal<bool>,
    save: impl Fn() + 'static,
    modal: ui::ModalHost,
) -> AnyElement {
    let close = move || modal.close();
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, "", close)
            .width(512.0)
            .body(
                view()
                    .p(px(24.0))
                    .gap(px(16.0))
                    .child(ui::required_field("Name", name, "Main account"))
                    .child(entity_icon_field(icon, color))
                    .child(
                        view()
                            .flex_col()
                            .gap(px(6.0))
                            .child(ui::label("Color"))
                            .child(crate::components::color_picker::color_picker(color)),
                    )
                    .child(
                        when(
                            move || !error.get().is_empty(),
                            move || {
                                text(move || error.get())
                                    .text_size(px(12.0))
                                    .text_color(move || theme().destructive)
                                    .into_element()
                            },
                        )
                        .otherwise(|| view().h(px(0.0)).into_element()),
                    )
                    .into_element(),
            )
            .footer(
                view()
                    .w_full()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.0))
                    .child(dialog_button("Cancel", false, close))
                    .child(dialog_button(
                        if busy.get() { "Saving..." } else { "Save" },
                        true,
                        save,
                    ))
                    .into_element(),
            ),
    )
    .into_element()
}

fn category_dialog(
    title: &'static str,
    name: Signal<String>,
    icon: Signal<String>,
    color: Signal<String>,
    error: Signal<String>,
    busy: Signal<bool>,
    save: impl Fn() + 'static,
    modal: ui::ModalHost,
) -> AnyElement {
    let close = move || modal.close();
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, "", close)
            .width(512.0)
            .body(
                view()
                    .p(px(24.0))
                    .gap(px(16.0))
                    .child(ui::required_field("Name", name, "Entertainment"))
                    .child(entity_icon_field(icon, color))
                    .child(
                        view()
                            .flex_col()
                            .gap(px(6.0))
                            .child(ui::label("Color"))
                            .child(crate::components::color_picker::color_picker(color)),
                    )
                    .child(
                        when(
                            move || !error.get().is_empty(),
                            move || {
                                text(move || error.get())
                                    .text_size(px(12.0))
                                    .text_color(move || theme().destructive)
                                    .into_element()
                            },
                        )
                        .otherwise(|| view().h(px(0.0)).into_element()),
                    )
                    .into_element(),
            )
            .footer(
                view()
                    .w_full()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.0))
                    .child(dialog_button("Cancel", false, close))
                    .child(dialog_button(
                        if busy.get() { "Saving..." } else { "Save" },
                        true,
                        save,
                    ))
                    .into_element(),
            ),
    )
    .into_element()
}

fn dialog_button(
    label: impl Into<String>,
    primary: bool,
    action: impl Fn() + 'static,
) -> impl IntoElement {
    let label = label.into();
    button()
        .cursor_pointer()
        .h(px(40.0))
        .px(px(16.0))
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || {
            if primary {
                theme().primary
            } else {
                theme().card
            }
        })
        .on_press(action)
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if primary {
                        theme().primary_foreground
                    } else {
                        theme().foreground
                    }
                }),
        )
}

fn admin_stat(
    label: &'static str,
    value: impl Fn() -> String + 'static,
    mono: bool,
    small: bool,
) -> AnyElement {
    ui::surface()
        .p(px(16.0))
        .gap(px(8.0))
        .child(
            text(label)
                .text_size(px(11.0))
                .letter_spacing(px(1.4))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            text(value)
                .text_size(if small { px(14.0) } else { px(24.0) })
                .line_height(if small { px(20.0) } else { px(32.0) })
                .font_family(if mono {
                    crate::theme::MONO
                } else {
                    crate::theme::SANS
                })
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .into_element()
}

fn admin_card(server: String, token: String, modal: ui::ModalHost) -> impl IntoElement {
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_admin_users(server, token).await }
        }
    });
    let settings_data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_admin_settings(server, token).await }
        }
    });
    let backup_status = signal(String::new());
    let registration = signal(false);
    let registration_loaded = signal(false);
    let last_registration = signal(false);
    effect(move || {
        if !registration_loaded.get()
            && let Some(settings) = settings_data.get()
        {
            registration.set(settings.registration_enabled);
            last_registration.set(settings.registration_enabled);
            registration_loaded.set(true);
        }
    });
    effect({
        let server = server.clone();
        let token = token.clone();
        move || {
            let value = registration.get();
            if !registration_loaded.get() || value == last_registration.get() {
                return;
            }
            last_registration.set(value);
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                let _ = crate::api::update_admin_settings(server, token, value).await;
                settings_data.refetch();
            });
        }
    });
    let remove_user = {
        let server = server.clone();
        let token = token.clone();
        move |id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                let _ = crate::api::delete_admin_user(server, token, id).await;
                data.refetch();
            });
        }
    };
    let users = move || data.get().unwrap_or_default();
    let user_count = move || users().len().to_string();
    let admin_count = move || {
        users()
            .iter()
            .filter(|user| user.is_admin)
            .count()
            .to_string()
    };
    let registration_label = move || {
        if registration.get() {
            "Open".to_string()
        } else {
            "Closed".to_string()
        }
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(3, minmax(0, 1fr))"))
                .gap(px(12.0))
                .child(admin_stat("USERS", user_count, true, false))
                .child(admin_stat("ADMINISTRATORS", admin_count, true, false))
                .child(admin_stat("REGISTRATION", registration_label, false, true)),
        )
        .child(
            ui::surface()
                .p(px(0.0))
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .flex_row()
                        .items_start()
                        .gap(px(12.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .child(
                            view()
                                .h(px(40.0))
                                .w(px(40.0))
                                .flex_shrink_0()
                                .items_center()
                                .justify_center()
                                .rounded(px(10.0))
                                .bg(move || theme().primary.opacity(0.1))
                                .child(
                                    image(asset!("icons/shield.svg"))
                                        .size(px(20.0))
                                        .tint_color(move || theme().primary),
                                ),
                        )
                        .child(
                            view()
                                .flex_col()
                                .child(
                                    text("Registration")
                                        .text_size(px(18.0))
                                        .line_height(px(28.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text("Control whether new accounts can be created from the login screen.")
                                        .text_size(px(14.0))
                                        .line_height(px(20.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .flex_row()
                        .items_start()
                        .justify_between()
                        .gap(px(16.0))
                        .child(
                            view()
                                .flex_row()
                                .items_start()
                                .gap(px(12.0))
                                .child(checkbox(registration, String::new()))
                                .child(
                                    view()
                                        .flex_col()
                                        .child(
                                            text("Allow new registrations")
                                                .text_size(px(14.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || theme().foreground),
                                        )
                                        .child(
                                            text("Existing accounts can always log in.")
                                                .text_size(px(12.0))
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                ),
                        ),
                ),
        )
        .child(data_card(server.clone(), token.clone(), backup_status, true))
        .child(
            ui::surface()
                .p(px(0.0))
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .child(
                            text("Users")
                                .text_size(px(18.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Accounts registered on this server.")
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .flex_col()
                        .child(each(
                            move || data.get().unwrap_or_default(),
                            |user| user.id,
                            {
                                let role_server = server.clone();
                                let role_token = token.clone();
                                move |user, _| {
                                    let user: crate::models::AdminUser = user.get();
                                    let id = user.id;
                                    let is_admin = user.is_admin;
                                    let email = user.email.clone();
                                    let name = user
                                        .name
                                        .clone()
                                        .filter(|value| !value.trim().is_empty())
                                        .unwrap_or_else(|| "No name".to_string());
                                    let joined = user
                                        .created_at
                                        .map(|value| value.format("%Y-%m-%d").to_string())
                                        .unwrap_or_default();
                                    let role_server = role_server.clone();
                                    let role_token = role_token.clone();
                                    let role = move || {
                                        let server = role_server.clone();
                                        let token = role_token.clone();
                                        spawn(async move {
                                            let _ = crate::api::update_admin_role(
                                                server, token, id, !is_admin,
                                            )
                                            .await;
                                            data.refetch();
                                        });
                                    };
                                    view()
                                        .w_full()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(12.0))
                                        .p(px(16.0))
                                        .border_b(px(1.0))
                                        .border_color(move || theme().border.opacity(0.6))
                                        .child(
                                            view()
                                                .h(px(36.0))
                                                .w(px(36.0))
                                                .flex_shrink_0()
                                                .items_center()
                                                .justify_center()
                                                .rounded(px(999.0))
                                                .bg(move || theme().primary.opacity(0.1))
                                                .child(
                                                    image(asset!("icons/shield.svg"))
                                                        .size(px(16.0))
                                                        .tint_color(move || theme().primary),
                                                ),
                                        )
                                        .child(
                                            view()
                                                .flex_grow(1.0)
                                                .basis(px(0.0))
                                                .min_w(px(0.0))
                                                .child(
                                                    text(email.clone())
                                                        .number_of_lines(1)
                                                        .text_size(px(14.0))
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(move || theme().foreground),
                                                )
                                                .child(
                                                    text(if joined.is_empty() {
                                                        name
                                                    } else {
                                                        format!("{name} · {joined}")
                                                    })
                                                    .number_of_lines(1)
                                                    .text_size(px(12.0))
                                                    .text_color(move || theme().muted_foreground),
                                                ),
                                        )
                                        .when(is_admin, |row| {
                                            row.child(badge(
                                                BadgeProps::new("Admin").tone(BadgeTone::Primary),
                                            ))
                                        })
                                        .child(badge(
                                            BadgeProps::new(if user.verified {
                                                "Verified"
                                            } else {
                                                "Unverified"
                                            })
                                            .tone(if user.verified {
                                                BadgeTone::Success
                                            } else {
                                                BadgeTone::Neutral
                                            }),
                                        ))
                                        .child(crate::components::button::button(
                                            crate::components::button::ButtonProps::new(
                                                if is_admin {
                                                    "Remove admin"
                                                } else {
                                                    "Make admin"
                                                },
                                            )
                                            .variant(crate::components::button::ButtonVariant::Outline)
                                            .size(crate::components::button::ButtonSize::Sm)
                                            .icon(asset!("icons/shield.svg"))
                                            .on_press(role),
                                        ))
                                        .child(ui::row_icon_button_labeled(
                                            asset!("icons/trash.svg"),
                                            "Delete user",
                                            true,
                                            {
                                                let email = email.clone();
                                                let delete = remove_user.clone();
                                                move || {
                                                    let email = email.clone();
                                                    let delete = delete.clone();
                                                    modal.open(move || {
                                                        let delete = delete.clone();
                                                        crate::components::dialog::dialog(
                                                            crate::components::dialog::DialogProps::new(
                                                                format!("Delete {email}?"),
                                                                "This permanently deletes the user and all of their accounts, categories, subscriptions, transactions, debts and settings.",
                                                                move || modal.close(),
                                                            )
                                                            .width(420.0)
                                                            .footer(
                                                                view()
                                                                    .w_full()
                                                                    .flex_row()
                                                                    .justify_end()
                                                                    .gap(px(8.0))
                                                                    .child(crate::components::button::button(
                                                                        crate::components::button::ButtonProps::new("Delete user")
                                                                            .variant(crate::components::button::ButtonVariant::Destructive)
                                                                            .size(crate::components::button::ButtonSize::Sm)
                                                                            .on_press(move || {
                                                                                delete(id);
                                                                                modal.close();
                                                                            }),
                                                                    ))
                                                                    .child(crate::components::button::button(
                                                                        crate::components::button::ButtonProps::new("Cancel")
                                                                            .variant(crate::components::button::ButtonVariant::Outline)
                                                                            .size(crate::components::button::ButtonSize::Sm)
                                                                            .on_press(move || modal.close()),
                                                                    ))
                                                                    .into_element(),
                                                            )
                                                        )
                                                        .into_element()
                                                    });
                                                }
                                            },
                                        ))
                                        .into_element()
                                }
                            },
                        )),
                ),
        )
        .into_element()
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
    fn avatar_data_url_resizes_and_encodes_the_photo() {
        let source = image::RgbImage::from_pixel(4, 4, image::Rgb([20, 40, 60]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(source)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("fixture image encodes");
        let data_url = avatar_data_url(&bytes).expect("avatar encodes");
        assert!(data_url.starts_with("data:image/jpeg;base64,"));
        assert!(data_url.len() <= 400_000);
    }

    #[test]
    fn avatar_rejects_non_images() {
        assert!(avatar_data_url(b"not an image").is_err());
    }

    #[test]
    fn update_request_skips_empty_fields() {
        let request = UpdateSettingsRequest {
            name: None,
            email: None,
            avatar: None,
            currency: None,
        };
        let value = serde_json::to_value(request).expect("settings serializes");
        assert_eq!(value.as_object().expect("object").len(), 0);
    }
}
