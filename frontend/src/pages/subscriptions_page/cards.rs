use super::*;

// ── Grid card ────────────────────────────────────────────────────────

pub fn render_grid_card(
    sub: &Subscription,
    index: usize,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
) -> Element {
    let opacity = subscription_card_opacity(sub.is_active);
    let formatted = format_currency(sub.amount, &sub.currency);
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &PREFERENCES().date_format());

    let (color, cat_icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".to_string()),
                c.icon.clone().unwrap_or_else(|| "Tag".to_string()),
                c.name.clone(),
            )
        } else {
            ("#6b7280".to_string(), "Tag".to_string(), "Uncategorized".to_string())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let has_notes = subscription_has_notes(&sub.notes);

    let stagger_delay = index.saturating_mul(60);

    rsx! {
        div {
            class: "group relative overflow-hidden rounded-xl border border-border/40 bg-card/80 hover:bg-card hover:scale-[1.02] transition-all duration-300 {opacity}",
            style: "animation: fadeSlideIn 0.5s ease-out both; animation-delay: {stagger_delay}ms;",

            // Category color top accent strip
            div {
                class: "absolute top-0 left-0 right-0 h-[3px]",
                style: "background: {color};",
            }

            // Logo watermark background
            if let Some(ref url) = logo_url {
                div { class: "absolute inset-0 opacity-[0.06] flex items-center justify-center pointer-events-none select-none",
                    img {
                        src: url.to_string(),
                        alt: "",
                        class: "w-full h-full object-contain scale-[2.5]",
                    }
                }
            }

            // Content
            div { class: "relative z-10 flex flex-col p-5 pb-4 gap-3",

                // Top row: logo + status badge
                div { class: "flex items-start justify-between gap-2",
                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-muted overflow-hidden",
                        {logo_image_with_fallback(&format!("card-{}", sub.id), logo_url.as_deref(), &name, "h-full w-full object-cover")}
                        if logo_url.is_none() {
                            span { class: "text-muted-foreground", CreditCard { size: 22 } }
                        }
                    }

                    // Status badge — square rounded-md
                    span {
                        class: if is_active { "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" } else { "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-muted text-muted-foreground border border-border/40" },
                        if is_active { "Active" } else { "Inactive" }
                    }
                }

                // Name + next billing
                div { class: "min-w-0",
                    p { class: "font-semibold truncate text-sm leading-tight", {name.to_string()} }
                    p { class: "text-xs text-muted-foreground mt-1", {next_text.to_string()} }
                }

                // Amount
                p { class: "font-extrabold text-2xl tracking-tight", {formatted.to_string()} }

                // Category badge
                span {
                    class: "inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium w-fit",
                    style: "color: {color}; background-color: {color}15; border: 1px solid {color}25;",
                    {icon_display(&cat_icon)}
                    {cat_name.to_string()}                }

                // Action buttons
                div { class: "flex items-center gap-0.5 mt-auto pt-1",
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
                        Trash2 { size: 14 }
                    }
                }
            }
        }
    }
}

// ── List card ─────────────────────────────────────────────────────────

pub fn render_list_card(
    sub: &Subscription,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
) -> Element {
    let opacity = subscription_card_opacity(sub.is_active);
    let formatted = format_currency(sub.amount, &sub.currency);
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &PREFERENCES().date_format());

    let (color, cat_icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".into()),
                c.icon.clone().unwrap_or_else(|| "Tag".into()),
                c.name.clone(),
            )
        } else {
            ("#6b7280".into(), "Tag".into(), "Uncategorized".into())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let has_notes = subscription_has_notes(&sub.notes);

    rsx! {
        div { class: "rounded-lg border bg-card {opacity} min-w-0",
            div { class: "p-3 sm:p-4",
                div { class: "flex items-center gap-2 sm:gap-4",
                    div { class: "flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden",
                        {logo_image_with_fallback(&format!("list-{}", sub.id), logo_url.as_deref(), &name, "h-full w-full object-cover")}
                        if logo_url.is_none() {
                            span { class: "text-muted-foreground", CreditCard { size: 20 } }
                        }
                    }

                    div { class: "flex-1 min-w-0",
                        p { class: "font-medium truncate text-sm sm:text-base", {name.to_string()} }
                        p { class: "text-xs text-muted-foreground truncate", {next_text.to_string()} }
                    }

                    div { class: "flex items-center gap-2 sm:gap-4 shrink-0",
                        span {
                            class: "hidden sm:inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-normal",
                            style: "color: {color}; background-color: {color}15;",
                            {icon_display(&cat_icon)}
                            {cat_name.to_string()}                        }
                        p { class: "font-medium text-sm sm:text-base", {formatted.to_string()} }

                        div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                            Tooltip { content: sub.notes.clone().unwrap_or_default(),
                                button {
                                    class: if has_notes { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground" } else { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground opacity-40" },
                                    onclick: move |_| on_notes.call(sub_for_notes.clone()),
                                    FileText { size: 16 }
                                }
                            }
                            button {
                                class: "h-7 w-7 sm:h-8 sm:w-8 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                                if is_active { Pause { size: 14 } } else { Play { size: 14 } }
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_edit.call(sub_for_edit.clone()),
                                Pencil { size: 14 }
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-red-600",
                                onclick: move |_| on_delete.call(id_for_delete),
                                Trash2 { size: 14 }
                            }
                        }
                    }
                }
            }
        }
    }
}

