use crate::state::preferences::PREFERENCES;
use crate::components::button::{Button, ButtonRadius, ButtonSize, ButtonVariant};
use crate::theme::catalog::{APPEARANCE_GROUPS, THEMES, ThemeDefinition, theme_by_id};
use crate::theme::hooks::apply_theme_class;
use dioxus::prelude::*;

/// Apply a catalogue theme locally and persist it on this device.
pub fn apply_theme(id: &str) {
    let Some(theme) = theme_by_id(id) else {
        return;
    };
    PREFERENCES.write().set_theme(theme.id.to_string());
    apply_theme_class(theme.id);
}

#[component]
fn ThemeSwatch(theme: ThemeDefinition) -> Element {
    rsx! {
        div {
            class: "flex h-7 overflow-hidden rounded-md border border-black/10 dark:border-white/10",
            aria_hidden: "true",
            span { class: "flex-1", style: "background-color: {theme.background};" }
            span { class: "flex-1", style: "background-color: {theme.card};" }
            span { class: "w-5", style: "background-color: {theme.primary};" }
        }
    }
}

#[component]
fn ThemeOption(theme: ThemeDefinition, selected: bool) -> Element {
    let select = move |_| apply_theme(theme.id);
    rsx! {
        Button {
            variant: ButtonVariant::Unstyled,
            size: ButtonSize::Unstyled,
            radius: ButtonRadius::None,
            class: (if selected {
                "group relative flex flex-col gap-2 rounded-lg border border-primary bg-primary/5 p-3 text-left transition-colors"
            } else {
                "group relative flex flex-col gap-2 rounded-lg border border-border/60 bg-background/30 p-3 text-left transition-colors hover:border-border hover:bg-accent/40"
            }).to_string(),
            aria_pressed: Some((if selected { "true" } else { "false" }).to_string()),
            onclick: select,
            ThemeSwatch { theme }
            div { class: "flex items-start justify-between gap-2",
                div { class: "min-w-0",
                    p { class: "truncate text-sm font-medium text-foreground", {theme.label} }
                    p { class: "mt-0.5 truncate text-xs text-muted-foreground", {theme.description} }
                }
                if selected {
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "mt-0.5 h-4 w-4 shrink-0 text-primary",
                        polyline { points: "20 6 9 17 4 12" }
                    }
                }
            }
        }
    }
}

/// Full theme picker used by Settings → General.
#[component]
pub fn ThemeGrid() -> Element {
    let current = PREFERENCES().theme().to_string();
    rsx! {
        div { class: "space-y-5",
            for (label, is_dark) in APPEARANCE_GROUPS {
                section { aria_label: "{label} themes",
                    h3 { class: "mb-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground/60",
                        {label}
                    }
                    div { class: "grid gap-2 sm:grid-cols-2 xl:grid-cols-3",
                        for theme in THEMES.iter().filter(|theme| theme.is_dark == is_dark) {
                            ThemeOption {
                                key: "{theme.id}",
                                theme: *theme,
                                selected: current == theme.id,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compact popover used by the top-bar theme button.
#[component]
pub fn ThemeMenu(on_close: EventHandler<()>) -> Element {
    let current = PREFERENCES().theme().to_string();
    let persistence_label = if crate::api::client::is_offline_mode() {
        "Saved on this device"
    } else {
        "Synced across devices"
    };
    rsx! {
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| on_close.call(()),
        }
        div {
            class: "fixed left-4 right-4 top-16 z-50 max-h-[70vh] overflow-y-auto rounded-lg border border-border bg-popover p-2 shadow-xl md:left-auto md:right-4 md:w-96",
            role: "menu",
            aria_label: "Color theme",
            div { class: "flex items-center justify-between px-2 py-1.5",
                p { class: "text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground/70", "Theme" }
                p { class: "text-[10px] text-muted-foreground/60", {persistence_label} }
            }
            div { class: "space-y-1",
                for (label, is_dark) in APPEARANCE_GROUPS {
                    p { class: "px-2 pt-2 text-[9px] font-semibold uppercase tracking-[0.16em] text-muted-foreground/60",
                        {label}
                    }
                    div { class: "grid grid-cols-2 gap-1",
                        for theme in THEMES.iter().filter(|theme| theme.is_dark == is_dark) {
                            {
                                let selected = current == theme.id;
                                let id = theme.id;
                                rsx! {
                                    Button {
                                        key: "{theme.id}",
                                        variant: ButtonVariant::Unstyled,
                                        size: ButtonSize::Unstyled,
                                        role: Some("menuitemradio".to_string()),
                                        onclick: move |_| {
                                            apply_theme(id);
                                            on_close.call(());
                                        },
                                        class: if selected {
                                            "flex items-center gap-2 rounded-md bg-primary/10 px-2 py-2 text-left ring-1 ring-primary/40"
                                        } else {
                                            "flex items-center gap-2 rounded-md px-2 py-2 text-left hover:bg-accent"
                                        },
                                        span {
                                            class: "flex h-5 w-8 shrink-0 overflow-hidden rounded border border-black/10 dark:border-white/10",
                                            aria_hidden: "true",
                                            span { class: "flex-1", style: "background-color: {theme.background};" }
                                            span { class: "flex-1", style: "background-color: {theme.card};" }
                                            span { class: "w-2", style: "background-color: {theme.primary};" }
                                        }
                                        span { class: "truncate text-xs font-medium", {theme.label} }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applying_unknown_theme_is_a_no_op() {
        apply_theme("not-a-theme");
    }

    #[test]
    fn catalogue_has_a_mocha_bloom_variant() {
        assert!(theme_by_id("mocha-bloom").is_some());
    }
}
