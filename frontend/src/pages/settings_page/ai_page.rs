use dioxus::prelude::*;
use lucide_dioxus::{Bot, KeyRound, Mic, ShieldCheck, Sparkles};

use crate::pages::settings_page::{AiConnectionCard, AiVoiceCard};
use crate::state::ai::AI_SETTINGS;
use crate::state::voice::{self};

#[component]
pub fn SettingsAiPage() -> Element {
    let settings = AI_SETTINGS.read().clone();
    let enabled = settings.as_ref().is_some_and(|settings| settings.enabled);
    let configured = settings
        .as_ref()
        .is_some_and(|settings| settings.configured);
    let voice_mode = *voice::VOICE_MODE.read();

    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            div { class: "relative overflow-hidden rounded-2xl border border-violet-500/20 bg-gradient-to-br from-violet-500/10 via-card/80 to-sky-500/5 p-6 sm:p-8",
                div { class: "pointer-events-none absolute -right-20 -top-24 h-72 w-72 rounded-full bg-violet-500/15 blur-3xl" }
                div { class: "pointer-events-none absolute -bottom-32 left-1/3 h-64 w-64 rounded-full bg-sky-500/10 blur-3xl" }
                div { class: "relative flex flex-wrap items-start gap-5",
                    span { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-violet-500/15 text-violet-600 shadow-sm ring-1 ring-violet-500/25 dark:text-violet-300",
                        Bot { size: 28 }
                    }
                    div { class: "min-w-0 flex-1",
                        div { class: "flex flex-wrap items-center gap-2",
                            h1 { class: "text-2xl font-bold tracking-tight", "AI assistant" }
                            span { class: "inline-flex items-center gap-1 rounded-full border border-violet-500/20 bg-violet-500/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-violet-700 dark:text-violet-300",
                                KeyRound { size: 11 }
                                "BYOK"
                            }
                        }
                        p { class: "mt-2 max-w-2xl text-sm text-muted-foreground",
                            "Connect an OpenAI-compatible model to create and change your subscriptions, transactions and debts. Choose your provider, model and how you want to dictate."
                        }
                        div { class: "mt-4 flex flex-wrap gap-2",
                            div { class: "inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-background/60 px-2.5 py-1 text-[11px] text-muted-foreground",
                                span { class: if configured && enabled { "h-1.5 w-1.5 rounded-full bg-emerald-500" } else { "h-1.5 w-1.5 rounded-full bg-muted-foreground/40" } }
                                if configured && enabled { "Assistant ready" } else if configured { "Assistant disabled" } else { "No key configured" }
                            }
                            div { class: "inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-background/60 px-2.5 py-1 text-[11px] text-muted-foreground",
                                Mic { size: 11 }
                                {voice_mode.label()}
                            }
                            div { class: "inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-background/60 px-2.5 py-1 text-[11px] text-muted-foreground",
                                ShieldCheck { size: 11 }
                                "Key encrypted at rest"
                            }
                        }
                    }
                }
            }

            AiConnectionCard {}
            AiVoiceCard {}

            div { class: "flex items-start gap-3 rounded-xl border border-border/40 bg-muted/20 p-4 text-xs text-muted-foreground",
                Sparkles { size: 14 }
                div {
                    p { class: "font-medium text-foreground", "Works with your data" }
                    p { class: "mt-0.5",
                        "The assistant receives a compact snapshot of your subscriptions, transactions, debts, accounts and categories. Actions run with your permissions."
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
    fn ai_page_has_a_hero_and_both_cards() {
        let source = include_str!("ai_page.rs");
        assert!(source.contains("AI assistant"));
        assert!(source.contains("AiConnectionCard"));
        assert!(source.contains("AiVoiceCard"));
        assert!(source.contains("BYOK"));
    }

    #[test]
    fn voice_mode_is_shown_in_the_hero() {
        let source = include_str!("ai_page.rs");
        assert!(source.contains("voice_mode.label()"));
        assert!(source.contains("Assistant ready"));
    }
}
