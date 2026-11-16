use dioxus::prelude::*;
use lucide_dioxus::{
    Cloud, Cpu, Download, Globe, HardDrive, Mic, ShieldCheck, Smartphone, Sparkles, Trash,
};

use crate::components::toast::TOAST;
use crate::state::ai::is_configured;
use crate::state::voice::{self, VoiceMode};
use crate::utils::local_speech::{self, LocalModel, MODELS};

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

fn mode_icon(mode: VoiceMode) -> Element {
    match mode {
        VoiceMode::Auto => rsx! { Sparkles { size: 18 } },
        VoiceMode::Native => rsx! { Smartphone { size: 18 } },
        VoiceMode::Browser => rsx! { Globe { size: 18 } },
        VoiceMode::Provider => rsx! { Cloud { size: 18 } },
        VoiceMode::Local => rsx! { Cpu { size: 18 } },
    }
}

fn mode_available(mode: VoiceMode) -> bool {
    match mode {
        VoiceMode::Auto => true,
        VoiceMode::Native => cfg!(target_os = "android"),
        VoiceMode::Browser => true,
        VoiceMode::Provider => is_configured(),
        VoiceMode::Local => local_speech::available(),
    }
}

fn mode_hint(mode: VoiceMode) -> &'static str {
    match mode {
        VoiceMode::Native if !cfg!(target_os = "android") => "Android only",
        VoiceMode::Local if !local_speech::available() => "Desktop app only",
        VoiceMode::Provider if !is_configured() => "Configure a model first",
        _ => "",
    }
}

fn gb(bytes: u64) -> f64 {
    bytes as f64 / GIB
}

#[component]
pub fn AiVoiceCard() -> Element {
    let mut mode = use_signal(|| *voice::VOICE_MODE.read());
    let (ram, cores) = local_speech::hardware();
    let recommended = local_speech::recommended_for(ram, cores);
    let mut selected_model = use_signal(|| {
        let stored = voice::VOICE_MODEL.read().clone();
        if local_speech::find(&stored).is_some() {
            stored
        } else {
            recommended.id.to_string()
        }
    });
    let mut downloading = use_signal(|| false);
    let mut progress = use_signal(|| 0.0_f64);
    let mut download_error = use_signal(String::new);
    let mut busy_model = use_signal(String::new);
    let mut installed_revision = use_signal(|| 0_u32);

    let mut choose_mode = move |next: VoiceMode| {
        voice::set_mode(next);
        mode.set(next);
    };

    let mut start_download = move |model: &'static LocalModel| {
        if downloading() {
            return;
        }
        downloading.set(true);
        progress.set(0.0);
        download_error.set(String::new());
        busy_model.set(model.id.to_string());
        voice::set_model(model.id);
        selected_model.set(model.id.to_string());
        spawn(async move {
            match local_speech::download(model, |done, total| {
                if total > 0 {
                    progress.set(done as f64 / total as f64);
                }
            })
            .await
            {
                Ok(()) => TOAST.write().success(format!("{} is ready", model.name)),
                Err(error) => {
                    download_error.set(error.clone());
                    TOAST
                        .write()
                        .error(format!("Could not download model: {error}"));
                }
            }
            downloading.set(false);
            busy_model.set(String::new());
            installed_revision += 1;
        });
    };

    let _ = installed_revision();
    let model_is_installed = |model: &LocalModel| local_speech::installed(model);
    let selected = local_speech::find(&selected_model()).unwrap_or(recommended);
    let installed_selected = model_is_installed(selected);
    let downloading_this = downloading() && busy_model() == selected.id;

    rsx! {
        div { class: "relative overflow-hidden rounded-2xl border border-sky-500/20 bg-card/80 shadow-sm",
            div { class: "pointer-events-none absolute -right-20 -bottom-28 h-64 w-64 rounded-full bg-sky-500/10 blur-3xl" }
            div { class: "relative flex flex-wrap items-start gap-4 border-b border-border/60 p-6",
                span { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-sky-500/10 text-sky-600 ring-1 ring-sky-500/20 dark:text-sky-300",
                    Mic { size: 24 }
                }
                div { class: "min-w-0 flex-1",
                    h2 { class: "text-lg font-semibold", "Voice input" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Dictate into the AI chat. Choose the engine that fits this device."
                    }
                }
                span { class: "inline-flex items-center gap-1 rounded-full border border-border/60 bg-background/60 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground",
                    ShieldCheck { size: 11 }
                    "Private by choice"
                }
            }
            div { class: "relative space-y-5 p-6",
                div { class: "grid gap-2 sm:grid-cols-2",
                    for option in VoiceMode::ALL {
                        {
                            let available = mode_available(option);
                            let hint = mode_hint(option);
                            let active = mode() == option;
                            rsx! {
                                button {
                                    r#type: "button",
                                    disabled: !available,
                                    aria_pressed: if active { "true" } else { "false" },
                                    class: if active {
                                        "group flex items-start gap-3 rounded-xl border border-primary/40 bg-primary/5 p-3.5 text-left ring-1 ring-primary/20"
                                    } else if available {
                                        "group flex items-start gap-3 rounded-xl border border-border/60 bg-background/40 p-3.5 text-left transition-colors hover:border-primary/30 hover:bg-accent/30"
                                    } else {
                                        "group flex items-start gap-3 rounded-xl border border-border/40 bg-background/20 p-3.5 text-left opacity-55"
                                    },
                                    onclick: move |_| choose_mode(option),
                                    span { class: if active {
                                        "mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary"
                                    } else {
                                        "mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground"
                                    },
                                        {mode_icon(option)}
                                    }
                                    span { class: "min-w-0",
                                        span { class: "flex flex-wrap items-center gap-1.5 text-sm font-semibold",
                                            {option.label()}
                                            if option == VoiceMode::Auto {
                                                span { class: "rounded-full bg-emerald-500/10 px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-300", "Recommended" }
                                            }
                                        }
                                        span { class: "mt-0.5 block text-xs leading-snug text-muted-foreground",
                                            {option.description()}
                                        }
                                        if !hint.is_empty() {
                                            span { class: "mt-1 block text-[10px] font-medium text-amber-600 dark:text-amber-400", {hint} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if local_speech::available() {
                    div { class: "rounded-xl border border-border/60 bg-background/30 p-4",
                        div { class: "flex flex-wrap items-center justify-between gap-2",
                            div {
                                p { class: "flex items-center gap-2 text-sm font-semibold",
                                    HardDrive { size: 15 }
                                    "On-device speech model"
                                }
                                p { class: "mt-1 text-xs text-muted-foreground",
                                    {format!("This device: {:.0} GB RAM · {} CPU cores", gb(ram), cores)}
                                }
                                p { class: "text-xs text-muted-foreground",
                                    {local_speech::recommendation_reason(recommended, ram, cores)}
                                }
                            }
                            if !installed_selected {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-xs font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                                    disabled: downloading(),
                                    onclick: move |_| start_download(selected),
                                    Download { size: 13 }
                                    if downloading_this { "Downloading…" } else { "Download" }
                                }
                            } else {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-8 items-center gap-1.5 rounded-md border border-destructive/30 px-3 text-xs font-medium text-destructive hover:bg-destructive/10",
                                    onclick: move |_| {
                                        match local_speech::delete(selected) {
                                            Ok(()) => {
                                                installed_revision += 1;
                                                TOAST.write().success("Local speech model removed");
                                            }
                                            Err(error) => TOAST.write().error(error),
                                        }
                                    },
                                    Trash { size: 13 }
                                    "Remove"
                                }
                            }
                        }
                        div { class: "mt-4 grid gap-2 sm:grid-cols-3",
                            for model in MODELS {
                                {
                                    let active = selected.id == model.id;
                                    let is_recommended = recommended.id == model.id;
                                    let installed = model_is_installed(model);
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            class: if active {
                                                "rounded-lg border border-primary/40 bg-primary/5 p-3 text-left"
                                            } else {
                                                "rounded-lg border border-border/50 bg-card/50 p-3 text-left transition-colors hover:border-primary/25 hover:bg-accent/30"
                                            },
                                            onclick: move |_| {
                                                voice::set_model(model.id);
                                                selected_model.set(model.id.to_string());
                                            },
                                            div { class: "flex items-center justify-between gap-2",
                                                span { class: "text-sm font-semibold", {model.name} }
                                                if installed {
                                                    span { class: "h-2 w-2 rounded-full bg-emerald-500", title: "Installed" }
                                                }
                                            }
                                            p { class: "mt-1 text-[11px] leading-snug text-muted-foreground", {model.description} }
                                            div { class: "mt-2 flex flex-wrap gap-1",
                                                span { class: "rounded-full bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground", {model.size_label} }
                                                span { class: "rounded-full bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground", {model.speed} }
                                                if is_recommended {
                                                    span { class: "rounded-full bg-emerald-500/10 px-1.5 py-0.5 text-[9px] font-semibold text-emerald-700 dark:text-emerald-300", "For you" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if downloading_this {
                            div { class: "mt-3",
                                div { class: "h-1.5 overflow-hidden rounded-full bg-muted",
                                    div { class: "h-full rounded-full bg-primary transition-all", style: "width: {progress() * 100.0:.0}%" }
                                }
                                p { class: "mt-1.5 text-xs text-muted-foreground",
                                    "Downloading {selected.name}… {progress() * 100.0:.0}%"
                                }
                            }
                        }
                        if !download_error().is_empty() {
                            p { class: "mt-2 text-xs text-destructive", {download_error()} }
                        }
                        if installed_selected {
                            p { class: "mt-2 text-[11px] text-muted-foreground",
                                "Installed locally. Audio never leaves this device when this option is selected."
                            }
                        } else if mode() == VoiceMode::Local {
                            p { class: "mt-2 text-[11px] text-amber-600 dark:text-amber-400",
                                "Download a model before using on-device dictation."
                            }
                        }
                    }
                }

                div { class: "flex items-start gap-2 rounded-lg border border-border/40 bg-muted/20 p-3 text-[11px] text-muted-foreground",
                    Cpu { size: 13 }
                    "Provider transcription uses the same encrypted API key as chat. Browser and Android speech may use services provided by your operating system or browser."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn voice_card_exposes_every_transcription_path() {
        let source = include_str!("voice_card.rs");
        assert!(source.contains("VoiceMode::ALL"));
        assert!(source.contains("mode_icon"));
        assert!(source.contains("local_speech::download"));
        assert!(source.contains("Provider transcription"));
        assert!(source.contains("Android speech"));
        assert!(source.contains("Browser speech"));
    }

    #[test]
    fn voice_card_recommends_by_hardware() {
        let source = include_str!("voice_card.rs");
        assert!(source.contains("recommendation_reason"));
        assert!(source.contains("GB RAM"));
    }

    #[test]
    fn voice_card_keeps_audio_local_for_local_mode() {
        let source = include_str!("voice_card.rs");
        assert!(source.contains("Audio never leaves this device"));
        assert!(source.contains("Provider transcription"));
    }
}
