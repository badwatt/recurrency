use dioxus::prelude::*;
use lucide_dioxus::{Download, FileText, ShieldCheck, Trash, Upload};

use crate::api::admin::{self, AdminSettings, AdminUser};
use crate::components::dialog::Dialog;
use crate::components::page_header::PageHeader;
use crate::components::toast::TOAST;
use crate::state::auth::AUTH;

const ADMIN_BACKUP_FILENAME: &str = "recurrency-admin-backup.json";
const ADMIN_BACKUP_MIME: &str = "application/json";
const ADMIN_BACKUP_ACCEPT: &str = "application/json,.json";
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

    let mut save_registration = move |enabled: bool| {
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

    let download_backup = move |_| {
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

    let stage_restore = move |file: dioxus::html::FileData| {
        spawn(async move {
            match file.read_string().await {
                Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(value) => restore_pending.set(Some(value)),
                    Err(_) => TOAST
                        .write()
                        .error("That file is not a valid Recurrency full backup"),
                },
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not read backup: {error}")),
            }
        });
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
                div { class: "rounded-xl border border-border/40 bg-card/80 p-6",
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
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader {
                title: "Administration",
                subtitle: "Users, registration and full installation backups",
            }

            div { class: "grid gap-3 sm:grid-cols-3",
                div { class: "rounded-xl border border-border/40 bg-card/80 p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Users" }
                    p { class: "mt-2 font-mono text-2xl font-bold tabular-nums", "{user_count}" }
                }
                div { class: "rounded-xl border border-border/40 bg-card/80 p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Administrators" }
                    p { class: "mt-2 font-mono text-2xl font-bold tabular-nums", "{admin_count}" }
                }
                div { class: "rounded-xl border border-border/40 bg-card/80 p-4",
                    p { class: "text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground", "Registration" }
                    p { class: "mt-2 text-sm font-semibold", if registration_enabled { "Open" } else { "Closed" } }
                }
            }

            div { class: "rounded-xl border border-border/40 bg-card/80",
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
                    label { r#for: "admin-registration-enabled", class: "flex min-w-0 items-start gap-3",
                        input {
                            r#type: "checkbox",
                            id: "admin-registration-enabled",
                            checked: registration_enabled,
                            disabled: saving(),
                            onchange: move |event| save_registration(event.checked()),
                            class: "mt-0.5 h-4 w-4 accent-primary",
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

            div { class: "rounded-xl border border-border/40 bg-card/80 overflow-hidden",
                div { class: "flex items-start gap-3 border-b border-border p-6",
                    span { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400", Download { size: 20 } }
                    div {
                        h2 { class: "text-lg font-semibold", "Full installation backup" }
                        p { class: "text-sm text-muted-foreground",
                            "Download every table, including users, accounts, categories and settings."
                        }
                    }
                    button {
                        r#type: "button",
                        class: "ml-auto inline-flex h-9 shrink-0 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: backing_up(),
                        onclick: download_backup,
                        Download { size: 15 }
                        if backing_up() { "Preparing…" } else { "Download" }
                    }
                }
                div { class: "flex items-center gap-3 p-4 text-xs text-muted-foreground",
                    FileText { size: 16 }
                    span { "JSON snapshot, generated with the current app version." }
                }
            }

            div { class: "rounded-xl border border-border/40 bg-card/80",
                div { class: "flex items-start gap-3 border-b border-border p-6",
                    span { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400", Upload { size: 20 } }
                    div {
                        h2 { class: "text-lg font-semibold", "Restore full backup" }
                        p { class: "text-sm text-muted-foreground",
                            "Replace every server table with a full backup file. This cannot be undone."
                        }
                    }
                }
                div { class: "space-y-4 p-6",
                    label { class: "relative inline-flex h-9 cursor-pointer items-center gap-2 rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-accent",
                        Upload { size: 15 }
                        "Choose backup"
                        input {
                            r#type: "file",
                            accept: ADMIN_BACKUP_ACCEPT,
                            aria_label: "Choose full backup",
                            class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                            onchange: move |event| {
                                if let Some(file) = event.files().first().cloned() {
                                    stage_restore(file);
                                }
                            },
                        }
                    }
                    if let Some(_) = restore_pending() {
                        div { class: "rounded-lg border border-destructive/40 bg-destructive/10 p-4",
                            p { class: "text-sm font-medium text-destructive", "Restore this backup?" }
                            p { class: "mt-1 text-xs text-muted-foreground",
                                "All current users, financial data and server settings will be replaced."
                            }
                            div { class: "mt-3 flex flex-wrap gap-2",
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-8 items-center rounded-md bg-destructive px-3 text-xs font-medium text-white hover:bg-destructive/90 disabled:opacity-50",
                                    disabled: restoring(),
                                    onclick: confirm_restore,
                                    if restoring() { "Restoring…" } else { "Restore backup" }
                                }
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-8 items-center rounded-md border border-border bg-background px-3 text-xs font-medium hover:bg-muted",
                                    onclick: move |_| restore_pending.set(None),
                                    "Cancel"
                                }
                            }
                        }
                    }
                }
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
                        button {
                            r#type: "button",
                            class: "inline-flex h-9 items-center rounded-md bg-destructive px-3 text-sm font-medium text-white hover:bg-destructive/90 disabled:opacity-50",
                            disabled: deleting(),
                            onclick: confirm_delete,
                            if deleting() { "Deleting…" } else { "Delete user" }
                        }
                        button {
                            r#type: "button",
                            class: "inline-flex h-9 items-center rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-muted",
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
                span { class: "rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary", "Admin" }
            }
            if user.verified {
                span { class: "rounded-full bg-emerald-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-300", "Verified" }
            } else {
                span { class: "rounded-full bg-muted px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground", "Unverified" }
            }
            if can_change_role {
                button {
                    r#type: "button",
                    class: "inline-flex h-8 shrink-0 items-center rounded-md border border-border px-2.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50",
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
                button {
                    r#type: "button",
                    aria_label: "Delete user",
                    title: "Delete user",
                    class: "inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive",
                    onclick: move |_| on_delete.call(user.id),
                    Trash { size: 15 }
                }
            }
        }
    }
}
