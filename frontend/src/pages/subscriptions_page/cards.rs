use super::*;

// ── Grid card ────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn render_grid_card(
    sub: &Subscription,
    index: usize,
    selection_mode: bool,
    selected: bool,
    on_select: Callback<Uuid>,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
    on_duplicate: Callback<Subscription>,
) -> Element {
    let today = chrono::Utc::now().date_naive();
    let finalized = subscription_is_finalized(sub.is_active, sub.end_date, today);
    let opacity = subscription_card_opacity(sub.is_active, finalized);
    let date_format = PREFERENCES().date_format();
    let mut next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &date_format);
    if finalized {
        next_text = "No upcoming charges".to_string();
    }
    let end_text = subscription_end_text(sub.end_date, today, &date_format);
    let category = CATEGORIES().get_by_id(&sub.category_id).cloned();
    let linked_debt = DEBTS()
        .debts
        .iter()
        .find(|debt| debt.subscription_id == Some(sub.id))
        .cloned();
    let is_debt = linked_debt.is_some();
    let schedule_progress = if is_debt {
        None
    } else {
        sub.next_billing_date.and_then(|next| {
            crate::utils::recurrence::payment_progress_for_year(
                sub.start_date,
                &sub.recurrence,
                sub.end_date,
                next,
            )
        })
    };

    let (color, account_icon, account_name) = {
        let accounts = ACCOUNTS();
        if let Some(a) = accounts.get_by_id(&sub.account_id) {
            (
                a.color.clone().unwrap_or_else(|| "#6b7280".to_string()),
                a.icon.clone().unwrap_or_else(|| "Tag".to_string()),
                a.name.clone(),
            )
        } else {
            ("#6b7280".to_string(), "Tag".to_string(), "No account".to_string())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let id_for_select = sub.id;
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let sub_for_duplicate = sub.clone();
    let has_notes = subscription_has_notes(&sub.notes);
    let status_label = if !is_active {
        "Inactive"
    } else if finalized {
        "Finalized"
    } else {
        "Active"
    };
    let status_class = if is_active && !finalized {
        "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
    } else if finalized {
        "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20"
    } else {
        "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-muted text-muted-foreground border border-border/40"
    };

    let stagger_delay = index.saturating_mul(60);
    let card_border = if finalized {
        "border-amber-500/25"
    } else {
        "border-border/40"
    };

    rsx! {
        div {
            class: "group relative overflow-hidden rounded-xl border {card_border} bg-card/80 hover:bg-card hover:scale-[1.02] transition-all duration-300 {opacity}",
            style: "animation: fadeSlideIn 0.5s ease-out both; animation-delay: {stagger_delay}ms;",

            // Logo-derived ambient color wash. Inline styles are intentional:
            // Android WebView must apply the opacity/blur even when arbitrary
            // Tailwind classes are missing from an older cached stylesheet.
            if let Some(url) = logo_url.as_deref() {
                img {
                    src: url.to_string(),
                    alt: "",
                    aria_hidden: "true",
                    class: "pointer-events-none absolute inset-0 h-full w-full select-none object-cover",
                    style: "opacity:0.28; filter:blur(52px) saturate(1.7); transform:scale(1.35);",
                }
                div { class: "pointer-events-none absolute inset-0 bg-card/75" }
            }

            // Account color top accent strip
            div {
                class: "absolute top-0 left-0 right-0 h-[3px]",
                style: "background: {color};",
            }

            // Content
            div { class: "relative z-10 flex flex-col p-5 pb-4 gap-3",

                // Top row: logo + status badge
                div { class: "flex items-start justify-between gap-2",
                    div { class: "flex items-center gap-2",
                        if selection_mode {
                            input {
                                r#type: "checkbox",
                                checked: selected,
                                aria_label: "Select subscription",
                                class: "h-4 w-4 shrink-0 cursor-pointer accent-primary",
                                onchange: move |_| on_select.call(id_for_select),
                            }
                        }
                        div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl overflow-hidden",
                            {logo_image_with_fallback(&format!("card-{}", sub.id), logo_url.as_deref(), &name, "h-12 w-12 object-contain")}
                            if logo_url.is_none() {
                                span { class: "text-muted-foreground", CreditCard { size: 22 } }
                            }
                        }
                    }

                    div { class: "flex items-center gap-1",
                        if is_debt {
                            span { class: "inline-flex items-center rounded-md border border-primary/20 bg-primary/10 px-2 py-0.5 text-[11px] font-medium text-primary", "Debt" }
                        }
                        span { class: status_class, {status_label} }
                    }
                }

                // Name + next billing + optional end date
                div { class: "min-w-0",
                    p { class: "font-semibold truncate text-sm leading-tight", {name.to_string()} }
                    p { class: "text-xs text-muted-foreground mt-1", {next_text.to_string()} }
                    if let Some(end) = end_text {
                        p { class: "text-[11px] text-muted-foreground/80 mt-0.5", {end} }
                    }
                    if let Some((current, total)) = linked_debt.as_ref().and_then(|debt| debt.installment()) {
                        p { class: "text-[11px] font-medium text-primary mt-0.5", "Payment {current}/{total}" }
                    } else if let Some((current, total)) = schedule_progress {
                        p { class: "text-[11px] font-medium text-primary mt-0.5", "Payment {current}/{total}" }
                    }
                }

                // Amount
                p { class: "font-extrabold text-2xl tracking-tight",
                    ConvertedAmount { amount: sub.amount, currency: sub.currency.clone() }
                }

                // Account and category badges
                div { class: "flex flex-wrap items-center gap-1.5",
                    span {
                        class: "inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium w-fit",
                        style: "color: {color}; background-color: {color}15; border: 1px solid {color}25;",
                        {icon_display(&account_icon)}
                        {account_name.to_string()}
                    }
                    if let Some(cat) = category.filter(|c| !c.is_system) {
                        {let cat_color = cat.color.clone().unwrap_or_else(|| "#6b7280".to_string());
                         let cat_icon = cat.icon.clone().unwrap_or_else(|| "Tag".to_string());
                         rsx! {
                            span {
                                class: "inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium w-fit",
                                style: "color: {cat_color}; background-color: {cat_color}15; border: 1px solid {cat_color}25;",
                                {icon_display(&cat_icon)}
                                {cat.name.to_string()}
                            }
                         }}
                    }
                }

                // Action buttons
                div { class: "flex items-center gap-0.5 mt-auto pt-1",
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200",
                        title: "Duplicate",
                        aria_label: "Duplicate subscription",
                        onclick: move |_| on_duplicate.call(sub_for_duplicate.clone()),
                        Copy { size: 14 }
                    }
                    Tooltip { content: sub.notes.clone().unwrap_or_default(),
                        button {
                            class: if has_notes { "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200" } else { "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground opacity-40 transition-colors duration-200" },
                            onclick: move |_| on_notes.call(sub_for_notes.clone()),
                            FileText { size: 16 }
                        }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200",
                        onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                        if is_active { Pause { size: 14 } } else { Play { size: 14 } }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200",
                        onclick: move |_| on_edit.call(sub_for_edit.clone()),
                        Pencil { size: 14 }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-red-500/10 text-red-500 transition-colors duration-200",
                        onclick: move |_| on_delete.call(id_for_delete),
                        Trash { size: 14 }
                    }
                }
            }
        }
    }
}

// ── List card ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn render_list_card(
    sub: &Subscription,
    selection_mode: bool,
    selected: bool,
    on_select: Callback<Uuid>,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
    on_duplicate: Callback<Subscription>,
) -> Element {
    let today = chrono::Utc::now().date_naive();
    let finalized = subscription_is_finalized(sub.is_active, sub.end_date, today);
    let opacity = subscription_card_opacity(sub.is_active, finalized);
    let date_format = PREFERENCES().date_format();
    let mut next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &date_format);
    if finalized {
        next_text = "No upcoming charges".to_string();
    }
    let end_text = subscription_end_text(sub.end_date, today, &date_format);
    let category = CATEGORIES().get_by_id(&sub.category_id).cloned();
    let linked_debt = DEBTS()
        .debts
        .iter()
        .find(|debt| debt.subscription_id == Some(sub.id))
        .cloned();
    let is_debt = linked_debt.is_some();
    let schedule_progress = if is_debt {
        None
    } else {
        sub.next_billing_date.and_then(|next| {
            crate::utils::recurrence::payment_progress_for_year(
                sub.start_date,
                &sub.recurrence,
                sub.end_date,
                next,
            )
        })
    };

    let (color, account_icon, account_name) = {
        let accounts = ACCOUNTS();
        if let Some(a) = accounts.get_by_id(&sub.account_id) {
            (
                a.color.clone().unwrap_or_else(|| "#6b7280".into()),
                a.icon.clone().unwrap_or_else(|| "Tag".into()),
                a.name.clone(),
            )
        } else {
            ("#6b7280".into(), "Tag".into(), "No account".into())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let id_for_select = sub.id;
    let status_label = if !is_active {
        "Inactive"
    } else if finalized {
        "Finalized"
    } else {
        "Active"
    };
    let card_border = if finalized {
        "border-amber-500/25"
    } else {
        ""
    };
    let status_class = if is_active && !finalized {
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium bg-emerald-500/10 text-emerald-500"
    } else if finalized {
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium bg-amber-500/10 text-amber-500"
    } else {
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground"
    };

    rsx! {
        div { class: "rounded-lg border {card_border} bg-card {opacity} min-w-0",
            div { class: "p-3 sm:p-4",
                div { class: "flex items-center gap-2 sm:gap-4",
                    if selection_mode {
                        input {
                            r#type: "checkbox",
                            checked: selected,
                            aria_label: "Select subscription",
                            class: "h-4 w-4 shrink-0 cursor-pointer accent-primary",
                            onchange: move |_| on_select.call(id_for_select),
                        }
                    }
                    div { class: "flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full overflow-hidden",
                        {logo_image_with_fallback(&format!("list-{}", sub.id), logo_url.as_deref(), &name, "h-9 w-9 sm:h-10 sm:w-10 object-contain")}
                        if logo_url.is_none() {
                            span { class: "text-muted-foreground", CreditCard { size: 20 } }
                        }
                    }

                    div { class: "flex-1 min-w-0",
                        div { class: "flex items-center gap-2 min-w-0",
                            p { class: "font-medium truncate text-sm sm:text-base", {name.to_string()} }
                            if is_debt {
                                span { class: "shrink-0 rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary", "Debt" }
                            }
                            span { class: status_class, {status_label} }
                        }
                        p { class: "text-xs text-muted-foreground truncate", {next_text.to_string()} }
                        if let Some(end) = end_text {
                            p { class: "text-[11px] text-muted-foreground/80 truncate", {end} }
                        }
                        if let Some((current, total)) = linked_debt.as_ref().and_then(|debt| debt.installment()) {
                            p { class: "text-[11px] font-medium text-primary truncate", "Payment {current}/{total}" }
                        } else if let Some((current, total)) = schedule_progress {
                            p { class: "text-[11px] font-medium text-primary truncate", "Payment {current}/{total}" }
                        }
                        div { class: "mt-1 flex flex-wrap items-center gap-1.5",
                            span {
                                class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-normal",
                                style: "color: {color}; background-color: {color}15;",
                                {icon_display(&account_icon)}
                                {account_name.to_string()}
                            }
                            if let Some(cat) = category.filter(|c| !c.is_system) {
                                {let cat_color = cat.color.clone().unwrap_or_else(|| "#6b7280".to_string());
                                 let cat_icon = cat.icon.clone().unwrap_or_else(|| "Tag".to_string());
                                 rsx! {
                                    span {
                                        class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-normal",
                                        style: "color: {cat_color}; background-color: {cat_color}15;",
                                        {icon_display(&cat_icon)}
                                        {cat.name.to_string()}
                                    }
                                 }}
                            }
                        }
                    }

                    div { class: "hidden sm:flex items-center gap-4 shrink-0",
                        p { class: "font-medium text-base", ConvertedAmount { amount: sub.amount, currency: sub.currency.clone() } }
                        {render_list_actions(sub, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
                    }
                }

                // Mobile: actions get their own row so the name has the full width.
                div { class: "mt-3 flex items-center justify-between gap-2 border-t border-border/40 pt-2 sm:hidden",
                    p { class: "font-semibold text-base", ConvertedAmount { amount: sub.amount, currency: sub.currency.clone() } }
                    {render_list_actions(sub, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_list_actions(
    sub: &Subscription,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
    on_duplicate: Callback<Subscription>,
) -> Element {
    let is_active = sub.is_active;
    let has_notes = subscription_has_notes(&sub.notes);
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let sub_for_duplicate = sub.clone();

    rsx! {
        div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
            button {
                class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                title: "Duplicate",
                aria_label: "Duplicate subscription",
                onclick: move |_| on_duplicate.call(sub_for_duplicate.clone()),
                Copy { size: 14 }
            }
            Tooltip { content: sub.notes.clone().unwrap_or_default(),
                button {
                    class: if has_notes { "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground" } else { "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground opacity-40" },
                    onclick: move |_| on_notes.call(sub_for_notes.clone()),
                    FileText { size: 16 }
                }
            }
            button {
                class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                if is_active { Pause { size: 14 } } else { Play { size: 14 } }
            }
            button {
                class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                onclick: move |_| on_edit.call(sub_for_edit.clone()),
                Pencil { size: 14 }
            }
            button {
                class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-red-600",
                onclick: move |_| on_delete.call(id_for_delete),
                Trash { size: 14 }
            }
        }
    }
}

