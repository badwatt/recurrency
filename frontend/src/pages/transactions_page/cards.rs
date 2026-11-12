use super::*;

/// One transaction row/card, shared by the list and grid views.
#[component]
pub fn TransactionCard(
    txn: Transaction,
    selection_mode: bool,
    selected: bool,
    on_select: Callback<Uuid>,
    on_edit: Callback<Uuid>,
    on_delete: Callback<Uuid>,
    on_notes: Callback<Uuid>,
) -> Element {
    let accounts = ACCOUNTS();
    let account = accounts.get_by_id(&txn.account_id);
    let account_name = account.map(|a| a.name.clone()).unwrap_or_else(|| "No account".into());
    let account_icon = account
        .and_then(|a| a.icon.clone())
        .unwrap_or_else(|| "Tag".into());
    let account_color = account
        .and_then(|a| a.color.clone())
        .unwrap_or_else(|| "#6b7280".into());
    let bg = format!("{}20", account_color);
    let has_notes = txn.notes.as_ref().is_some_and(|notes| !notes.is_empty());
    let category = CATEGORIES().get_by_id(&txn.category_id).cloned();
    let id = txn.id;

    rsx! {
        div { key: "{txn.id}", class: "rounded-lg border border-border bg-card p-3 sm:p-4 hover:bg-muted/50 transition-colors",
            div { class: "flex items-center gap-2 sm:gap-4",
                if selection_mode {
                    input {
                        r#type: "checkbox",
                        checked: selected,
                        aria_label: "Select transaction",
                        class: "h-4 w-4 shrink-0 cursor-pointer accent-primary",
                        onchange: move |_| on_select.call(id),
                    }
                }
                div { class: "flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden",
                    {logo_image_with_fallback(&format!("txn-{}", txn.id), txn.logo_url.as_deref(), &txn.name, "h-9 w-9 sm:h-10 sm:w-10 object-contain p-1")}
                    if txn.logo_url.is_none() {
                        span { class: "text-muted-foreground", Receipt { size: 16 } }
                    }
                }
                div { class: "flex-1 min-w-0",
                    p { class: "font-medium truncate text-sm sm:text-base", {txn.name.clone()} }
                    p { class: "text-xs text-muted-foreground",
                        {format_date(&txn.date, &PREFERENCES().date_format())}
                    }
                    div { class: "mt-1 flex flex-wrap items-center gap-1.5",
                        span { class: "font-normal text-xs inline-flex items-center gap-1 px-2 py-0.5 rounded",
                            style: "color: {account_color}; background-color: {bg}",
                            {icon_display(&account_icon)}
                            {account_name}
                        }
                        if let Some(category) = category.clone().filter(|category| !category.is_system) {
                            {let color = category.color.clone().unwrap_or_else(|| "#6b7280".into());
                             let icon = category.icon.clone().unwrap_or_else(|| "Tag".into());
                             rsx! {
                                span { class: "font-normal text-xs inline-flex items-center gap-1 rounded-full px-2 py-0.5",
                                    style: "color: {color}; background-color: {color}15;",
                                    {icon_display(&icon)}
                                    {category.name.clone()}
                                }
                             }}
                        }
                    }
                }
                p { class: "font-medium text-sm sm:text-base shrink-0",
                    ConvertedAmount { amount: txn.amount, currency: txn.currency.clone() }
                }
                div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                    Tooltip { content: txn.notes.clone().unwrap_or_default(),
                        button {
                            class: if has_notes { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-muted-foreground" } else { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-muted-foreground opacity-40" },
                            aria_label: "Notes",
                            onclick: move |_| on_notes.call(id),
                            FileText { size: 16 }
                        }
                    }
                    button {
                        class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent",
                        aria_label: "Edit transaction",
                        onclick: move |_| on_edit.call(id),
                        Pencil { size: 16 }
                    }
                    button {
                        class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-destructive",
                        aria_label: "Delete transaction",
                        onclick: move |_| on_delete.call(id),
                        Trash { size: 16 }
                    }
                }
            }
        }
    }
}

/// Compact vertical card used by the transactions grid view.
#[component]
pub fn TransactionGridCard(
    txn: Transaction,
    selection_mode: bool,
    selected: bool,
    on_select: Callback<Uuid>,
    on_edit: Callback<Uuid>,
    on_delete: Callback<Uuid>,
    on_notes: Callback<Uuid>,
) -> Element {
    let accounts = ACCOUNTS();
    let account = accounts.get_by_id(&txn.account_id);
    let account_name = account.map(|a| a.name.clone()).unwrap_or_else(|| "No account".into());
    let account_icon = account
        .and_then(|a| a.icon.clone())
        .unwrap_or_else(|| "Tag".into());
    let account_color = account
        .and_then(|a| a.color.clone())
        .unwrap_or_else(|| "#6b7280".into());
    let bg = format!("{}20", account_color);
    let has_notes = txn.notes.as_ref().is_some_and(|notes| !notes.is_empty());
    let category = CATEGORIES().get_by_id(&txn.category_id).cloned();
    let id = txn.id;

    rsx! {
        div { key: "{txn.id}", class: "flex min-w-0 flex-col gap-3 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-muted/50",
            div { class: "flex min-w-0 items-start gap-3",
                if selection_mode {
                    input {
                        r#type: "checkbox",
                        checked: selected,
                        aria_label: "Select transaction",
                        class: "mt-2 h-4 w-4 shrink-0 cursor-pointer accent-primary",
                        onchange: move |_| on_select.call(id),
                    }
                }
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden",
                    {logo_image_with_fallback(&format!("txn-grid-{}", txn.id), txn.logo_url.as_deref(), &txn.name, "h-10 w-10 object-contain p-1")}
                    if txn.logo_url.is_none() {
                        span { class: "text-muted-foreground", Receipt { size: 18 } }
                    }
                }
                div { class: "min-w-0 flex-1",
                    p { class: "truncate font-medium", {txn.name.clone()} }
                    p { class: "text-xs text-muted-foreground", {format_date(&txn.date, &PREFERENCES().date_format())} }
                    div { class: "mt-1 flex flex-wrap items-center gap-1.5",
                        span { class: "inline-flex items-center gap-1 truncate rounded-full px-2 py-0.5 text-[11px]",
                            style: "color: {account_color}; background-color: {bg}",
                            {icon_display(&account_icon)}
                            span { class: "truncate", {account_name} }
                        }
                        if let Some(category) = category.clone().filter(|category| !category.is_system) {
                            {let color = category.color.clone().unwrap_or_else(|| "#6b7280".into());
                             let icon = category.icon.clone().unwrap_or_else(|| "Tag".into());
                             rsx! {
                                span { class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px]",
                                    style: "color: {color}; background-color: {color}15;",
                                    {icon_display(&icon)}
                                    {category.name.clone()}
                                }
                             }}
                        }
                    }
                }
            }
            p { class: "font-semibold",
                ConvertedAmount { amount: txn.amount, currency: txn.currency.clone() }
            }
            div { class: "flex items-center justify-end gap-1 border-t border-border/40 pt-2",
                Tooltip { content: txn.notes.clone().unwrap_or_default(),
                    button {
                        class: if has_notes { "inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent text-muted-foreground" } else { "inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent text-muted-foreground opacity-40" },
                        aria_label: "Notes",
                        onclick: move |_| on_notes.call(id),
                        FileText { size: 15 }
                    }
                }
                button {
                    class: "inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent",
                    aria_label: "Edit transaction",
                    onclick: move |_| on_edit.call(id),
                    Pencil { size: 15 }
                }
                button {
                    class: "inline-flex h-8 w-8 items-center justify-center rounded-md text-destructive hover:bg-accent",
                    aria_label: "Delete transaction",
                    onclick: move |_| on_delete.call(id),
                    Trash { size: 15 }
                }
            }
        }
    }
}
