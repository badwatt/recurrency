use dioxus::prelude::*;
use lucide_dioxus::{ShieldCheck, Trash};

use crate::api::admin::{self, AdminSettings, AdminUser};
use crate::components::backup_restore_card::BackupRestoreCard;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant, IconButton};
use crate::components::card::Card;
use crate::components::checkbox::Checkbox;
use crate::components::dialog::Dialog;
use crate::components::page_header::PageHeader;
use crate::components::toast::TOAST;
use crate::state::auth::AUTH;

const ADMIN_BACKUP_FILENAME: &str = "recurrency-admin-backup.json";
const ADMIN_BACKUP_MIME: &str = "application/json";
const ADMIN_BACKUP_DETAIL: &str = "Every table, including users, accounts and server settings";
const ADMIN_BACKUP_NOTE: &str = "Keep it private; it contains every server table.";
const ADMIN_RESTORE_WARNING: &str =
    "All current users, financial data and server settings will be replaced.";
const ADMIN_DATE_FORMAT: &str = "%Y-%m-%d";
const UNKNOWN_NAME: &str = "No name";

#[component]
pub fn AdminPage() -> Element {
    let is_admin = AUTH().user.as_ref().is_some_and(|user| user.is_admin);
    let mut users = use_signal(Vec::<AdminUser>::new);
    let mut settings = use_signal(|| None::<AdminSettings>);
    let mut loading = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut backing_up = use_signal(|| false);
    let mut restoring = use_signal(|| false);
    let mut restore_pending = use_signal(|| None::<serde_json::Value>);
    let mut deleting = use_signal(|| false);
    let mut delete_target = use_signal(|| None::<uuid::Uuid>);
    let mut changing_role = use_signal(|| None::<uuid::Uuid>);
    let current_user_id = AUTH().user.as_ref().map(|user| user.id);
    let nav = navigator();

    use_future(move || async move {
        if !is_admin {
            loading.set(false);
            return;
        }
        match admin::users().await {
            Ok(data) => users.set(data),
            Err(error) => TOAST
                .write()
                .error(format!("Could not load users: {error}")),
        }
        match admin::settings().await {
            Ok(data) => settings.set(Some(data)),
            Err(error) => TOAST
                .write()
                .error(format!("Could not load settings: {error}")),
        }
        loading.set(false);
    });

    let save_registration = move |enabled: bool| {
        if saving() {
            return;
        }
        saving.set(true);
        spawn(async move {
            match admin::set_registration_enabled(enabled).await {
                Ok(data) => {
                    settings.set(Some(data));
                    TOAST.write().success(if enabled {
                        "Registration enabled"
                    } else {
                        "Registration disabled"
                    });
                }
                Err(error) => {
                    TOAST
                        .write()
                        .error(format!("Could not update registration: {error}"));
                }
            }
            saving.set(false);
        });
    };

    let download_backup = move |_: ()| {
        if backing_up() {
            return;
        }
        backing_up.set(true);
        spawn(async move {
            match admin::full_backup().await {
                Ok(value) => match serde_json::to_string_pretty(&value) {
                    Ok(json) => {
                        crate::utils::download::save_text(
                            ADMIN_BACKUP_FILENAME,
                            ADMIN_BACKUP_MIME,
                            &json,
                        );
                        TOAST.write().success("Full backup download started");
                    }
                    Err(error) => TOAST
                        .write()
                        .error(format!("Could not encode full backup: {error}")),
                },
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not create full backup: {error}")),
            }
            backing_up.set(false);
        });
    };

    let stage_restore = move |text: String| {
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => restore_pending.set(Some(value)),
            Err(_) => TOAST
                .write()
                .error("That file is not a valid Recurrency full backup"),
        }
    };

    let confirm_restore = move |_| {
        let Some(backup) = restore_pending() else {
            return;
        };
        if restoring() {
            return;
        }
        restoring.set(true);
        spawn(async move {
            match admin::restore_backup(&backup).await {
                Ok(result) => {
                    TOAST.write().success(format!(
                        "Restored {} tables. Sign in again.",
                        result.restored_tables
                    ));
                    AUTH.write().logout();
                    AUTH.write().persist_to_storage();
                    nav.push(crate::Route::Login {});
                }
                Err(error) => {
                    TOAST
                        .write()
                        .error(format!("Could not restore backup: {error}"));
                }
            }
            restoring.set(false);
        });
    };

    let confirm_delete = move |_| {
        let Some(user_id) = delete_target() else {
            return;
        };
        if deleting() {
            return;
        }
        deleting.set(true);
        spawn(async move {
            match admin::delete_user(user_id).await {
                Ok(()) => {
                    users.write().retain(|user| user.id != user_id);
                    delete_target.set(None);
                    TOAST.write().success("User deleted");
                }
                Err(error) => {
                    TOAST
                        .write()
                        .error(format!("Could not delete user: {error}"));
                }
            }
            deleting.set(false);
        });
    };

    let toggle_admin = EventHandler::new(move |user_id: uuid::Uuid| {
        if changing_role() == Some(user_id) {
            return;
        }
        let Some(make_admin) = users()
            .iter()
            .find(|user| user.id == user_id)
            .map(|user| !user.is_admin)
        else {
            return;
        };
        changing_role.set(Some(user_id));
        spawn(async move {
            match admin::set_admin(user_id, make_admin).await {
                Ok(updated) => {
                    if let Some(user) = users.write().iter_mut().find(|user| user.id == user_id) {
                        *user = updated.clone();
                    }
                    if current_user_id == Some(user_id) {
                        let mut auth = AUTH.write();
                        if let Some(user) = auth.user.as_mut() {
                            user.is_admin = updated.is_admin;
                        }
                        auth.persist_to_storage();
                        if !updated.is_admin {
                            nav.push(crate::Route::SettingsProfile {});
                        }
                    }
                    TOAST.write().success(if updated.is_admin {
                        "User promoted to administrator"
                    } else {
                        "Administrator role removed"
                    });
                }
                Err(error) => {
                    TOAST
                        .write()
                        .error(format!("Could not update role: {error}"));
                }
            }
            changing_role.set(None);
        });
    });

    if !is_admin {
        return rsx! {
            div { class: "flex flex-col gap-6",
                PageHeader {
                    title: "Administration",
                    subtitle: "This area is restricted to administrators",
                }
                Card { class: "p-6",
                    p { class: "text-sm text-muted-foreground",
                        "Ask an existing administrator to grant this account access."
                    }
                }
            }
        };
    }

    let registration_enabled = settings()
        .map(|data| data.registration_enabled)
        .unwrap_or(true);
    let user_count = users().len();
    let admin_count = users().iter().filter(|user| user.is_admin).count();

    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader {
                title: "Administration",
                subtitle: "Users, registration and full installation backups",
            }

            div { class: "grid gap-3 sm:grid-cols-3",
                Card { class: "p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Users" }
                    p { class: "mt-2 font-mono text-2xl font-bold tabular-nums", "{user_count}" }
                }
                Card { class: "p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Administrators" }
                    p { class: "mt-2 font-mono text-2xl font-bold tabular-nums", "{admin_count}" }
                }
                Card { class: "p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Registration" }
                    p { class: "mt-2 text-sm font-semibold", if registration_enabled { "Open" } else { "Closed" } }
                }
            }

            Card {
                div { class: "flex items-start gap-3 border-b border-border p-6",
                    span { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary", ShieldCheck { size: 20 } }
                    div {
                        h2 { class: "text-lg font-semibold", "Registration" }
                        p { class: "text-sm text-muted-foreground",
                            "Control whether new accounts can be created from the login screen."
                        }
                    }
                }
                div { class: "flex items-center justify-between gap-4 p-6",
                    label { class: "flex min-w-0 items-start gap-3",
                        Checkbox {
                            id: "admin-registration-enabled",
                            checked: registration_enabled,
                            disabled: saving(),
                            on_change: save_registration,
                            class: "mt-0.5",
                        }
                        span {
                            span { class: "block text-sm font-medium", "Allow new registrations" }
                            span { class: "block text-xs text-muted-foreground",
                                "Existing accounts can always log in."
                            }
                        }
                    }
                    if saving() {
                        span { class: "text-xs text-muted-foreground", "Saving…" }
                    }
                }
            }

            BackupRestoreCard {
                file_name: ADMIN_BACKUP_FILENAME,
                file_detail: ADMIN_BACKUP_DETAIL,
                export_note: ADMIN_BACKUP_NOTE,
                import_warning: ADMIN_RESTORE_WARNING,
                pending: restore_pending().is_some(),
                exporting: backing_up(),
                restoring: restoring(),
                on_export: download_backup,
                on_text: stage_restore,
                on_confirm: confirm_restore,
                on_cancel: move |_| restore_pending.set(None),
            }

            if let Some(target) = delete_target().and_then(|id| users().into_iter().find(|user| user.id == id)) {
                Dialog {
                    open: true,
                    on_close: move |_| delete_target.set(None),
                    title: format!("Delete {}?", target.email),
                    p { class: "text-sm text-muted-foreground",
                        "This permanently deletes the user and all of their accounts, categories, subscriptions, transactions, debts and settings."
                    }
                    div { class: "mt-4 flex flex-wrap gap-2",
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            disabled: deleting(),
                            onclick: confirm_delete,
                            if deleting() { "Deleting…" } else { "Delete user" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| delete_target.set(None),
                            "Cancel"
                        }
                    }
                }
            }

            div { class: "rounded-xl border border-border/40 bg-card/80 overflow-hidden",
                div { class: "flex items-center justify-between gap-4 border-b border-border p-6",
                    div {
                        h2 { class: "text-lg font-semibold", "Users" }
                        p { class: "text-sm text-muted-foreground", "Accounts registered on this server." }
                    }
                    if loading() {
                        span { class: "text-xs text-muted-foreground", "Loading…" }
                    }
                }
                if users().is_empty() && !loading() {
                    p { class: "p-6 text-sm text-muted-foreground", "No users found." }
                } else {
                    div { class: "divide-y divide-border/60",
                        for user in users() {
                            {
                                let user_id = user.id;
                                let can_delete = current_user_id != Some(user_id)
                                    && !(user.is_admin && admin_count <= 1);
                                let can_change_role = !(user.is_admin && admin_count <= 1);
                                let is_changing_role = changing_role() == Some(user_id);
                                let on_toggle_admin = toggle_admin;
                                rsx! {
                                    UserRow {
                                        key: "{user.id}",
                                        user,
                                        can_delete,
                                        can_change_role,
                                        changing_role: is_changing_role,
                                        on_delete: move |id| delete_target.set(Some(id)),
                                        on_toggle_admin,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UserRow(
    user: AdminUser,
    can_delete: bool,
    can_change_role: bool,
    changing_role: bool,
    on_delete: EventHandler<uuid::Uuid>,
    on_toggle_admin: EventHandler<uuid::Uuid>,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 p-4",
            div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary",
                ShieldCheck { size: 16 }
            }
            div { class: "min-w-0 flex-1",
                p { class: "truncate text-sm font-medium", {user.email} }
                p { class: "truncate text-xs text-muted-foreground",
                    {user.name.unwrap_or_else(|| UNKNOWN_NAME.to_string())}
                    " · "
                    {user.created_at.format(ADMIN_DATE_FORMAT).to_string()}
                }
            }
            if user.is_admin {
                Badge { variant: BadgeVariant::Primary, class: "rounded-full uppercase tracking-wide", "Admin" }
            }
            if user.verified {
                Badge { variant: BadgeVariant::Success, class: "rounded-full uppercase tracking-wide", "Verified" }
            } else {
                Badge { variant: BadgeVariant::Muted, class: "rounded-full uppercase tracking-wide", "Unverified" }
            }
            if can_change_role {
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    class: "shrink-0 text-muted-foreground",
                    disabled: changing_role,
                    onclick: move |_| on_toggle_admin.call(user.id),
                    ShieldCheck { size: 14 }
                    if changing_role {
                        "Saving…"
                    } else if user.is_admin {
                        "Remove admin"
                    } else {
                        "Make admin"
                    }
                }
            }
            if can_delete {
                IconButton {
                    label: "Delete user".to_string(),
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::IconSm,
                    class: "shrink-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive".to_string(),
                    onclick: move |_| on_delete.call(user.id),
                    Trash { size: 15 }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn admin_backups_use_the_same_card_as_data_backup() {
        let source = include_str!("admin_page.rs");
        assert!(source.contains("BackupRestoreCard"));
        assert!(source.contains("ADMIN_BACKUP_FILENAME"));
        assert!(source.contains("ADMIN_RESTORE_WARNING"));
    }
}
