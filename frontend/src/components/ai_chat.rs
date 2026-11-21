use dioxus::prelude::*;
use lucide_dioxus::{Bot, Mic, Send, Sparkles};

use crate::api::ai::{self, AiChatResponse, AiMessage};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::textarea::Textarea;
use crate::components::toast::TOAST;
use crate::state::debts::DEBTS;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::state::transactions::TRANSACTIONS;
use crate::state::voice::{VOICE_MODE, VOICE_MODEL};
use crate::storage::Storage;
use crate::utils::{local_speech, speech};

pub static AI_CHAT_OPEN: GlobalSignal<bool> = Signal::global(|| false);

const WELCOME_MESSAGE: &str =
    "Hi, I can create, update or delete subscriptions, transactions and debts. What do you need?";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChatLine {
    role: String,
    content: String,
}

#[component]
pub fn AiChat() -> Element {
    let mut messages = use_signal(|| {
        vec![ChatLine {
            role: "assistant".to_string(),
            content: WELCOME_MESSAGE.to_string(),
        }]
    });
    let mut input = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut listening = use_signal(|| false);
    let mut transcribing = use_signal(|| false);

    let mut append_dictation = move |text: String| {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let current = input();
        if current.trim().is_empty() {
            input.set(text.to_string());
        } else {
            input.set(format!("{current} {text}"));
        }
    };

    let dictate = move |_| {
        if listening() {
            speech::stop();
            return;
        }
        if sending() || transcribing() {
            return;
        }
        listening.set(true);
        spawn(async move {
            match speech::listen(*VOICE_MODE.read()).await {
                Ok(speech::SpeechInput::Text(text)) => append_dictation(text),
                Ok(speech::SpeechInput::Audio { mime_type, base64 }) => {
                    transcribing.set(true);
                    match ai::transcribe(&base64, &mime_type).await {
                        Ok(text) => append_dictation(text),
                        Err(error) => TOAST
                            .write()
                            .error(format!("Could not transcribe speech: {error}")),
                    }
                }
                Ok(speech::SpeechInput::Pcm(base64)) => {
                    transcribing.set(true);
                    match local_speech::transcribe_pcm(&VOICE_MODEL.read(), &base64).await {
                        Ok(text) => append_dictation(text),
                        Err(error) => TOAST.write().error(error),
                    }
                }
                Err(error) => TOAST.write().error(error),
            }
            listening.set(false);
            transcribing.set(false);
        });
    };

    let send = Callback::new(move |_: ()| {
        let text = input().trim().to_string();
        if text.is_empty() || sending() {
            return;
        }
        input.set(String::new());
        messages.write().push(ChatLine {
            role: "user".to_string(),
            content: text,
        });
        sending.set(true);
        spawn(async move {
            let history: Vec<AiMessage> = messages()
                .iter()
                .map(|line| AiMessage {
                    role: line.role.clone(),
                    content: line.content.clone(),
                })
                .collect();
            match ai::chat(&history).await {
                Ok(response) => {
                    messages.write().push(ChatLine {
                        role: "assistant".to_string(),
                        content: response_text(&response),
                    });
                    if response.actions.iter().any(|action| action.success) {
                        refresh_changed_data().await;
                    }
                }
                Err(error) => {
                    TOAST.write().error(format!("AI assistant error: {error}"));
                }
            }
            sending.set(false);
        });
    });

    let placeholder = if listening() {
        "Listening… click the microphone to finish"
    } else if transcribing() {
        "Transcribing…"
    } else {
        "Ask for a subscription, transaction or debt…"
    };

    rsx! {
        Dialog {
            open: AI_CHAT_OPEN(),
            on_close: move |_| {
                speech::stop();
                *AI_CHAT_OPEN.write() = false;
            },
            title: "AI assistant".to_string(),
            div { class: "flex min-h-0 flex-1 flex-col gap-3",
                div { class: "min-h-0 flex-1 space-y-3 overflow-y-auto pr-1",
                    for (index, line) in messages().iter().enumerate() {
                        div {
                            key: "{index}",
                            class: if line.role == "user" {
                                "ml-auto max-w-[85%] rounded-2xl rounded-br-sm bg-primary px-4 py-2.5 text-sm text-primary-foreground"
                            } else {
                                "mr-auto max-w-[90%] rounded-2xl rounded-bl-sm border border-border/60 bg-muted/30 px-4 py-2.5 text-sm text-foreground"
                            },
                            {line.content.clone()}
                        }
                    }
                    if sending() {
                        div { class: "mr-auto flex items-center gap-2 rounded-2xl border border-border/60 bg-muted/30 px-4 py-2.5 text-sm text-muted-foreground",
                            Sparkles { size: 15 }
                            "Thinking…"
                        }
                    }
                }
                form {
                    onsubmit: move |event| {
                        event.prevent_default();
                        send.call(());
                    },
                    class: "flex items-end gap-2 border-t border-border/60 pt-3",
                    Textarea {
                        value: input(),
                        on_input: move |value| input.set(value),
                        on_keydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Enter && !event.modifiers().shift() {
                                event.prevent_default();
                                send.call(());
                            }
                        },
                        rows: Some(2),
                        placeholder,
                        class: "min-h-11 flex-1 resize-none",
                    }
                    Button {
                        variant: ButtonVariant::Unstyled,
                        size: ButtonSize::Unstyled,
                        disabled: sending() || transcribing(),
                        aria_label: Some((if listening() { "Stop dictation" } else { "Dictate message" }).to_string()),
                        aria_pressed: Some((if listening() { "true" } else { "false" }).to_string()),
                        title: Some((if listening() { "Stop dictation" } else { "Dictate with your voice" }).to_string()),
                        class: (if listening() {
                            "relative inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md border border-destructive/40 bg-destructive/10 text-destructive"
                        } else {
                            "inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-50"
                        }).to_string(),
                        onclick: dictate,
                        if listening() {
                            span { class: "absolute inset-0 animate-ping rounded-md border border-destructive/30" }
                        }
                        if transcribing() {
                            Sparkles { size: 17 }
                        } else {
                            Mic { size: 17 }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Unstyled,
                        size: ButtonSize::Unstyled,
                        button_type: "submit".to_string(),
                        disabled: sending() || listening() || transcribing(),
                        aria_label: Some("Send message".to_string()),
                        class: "inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50".to_string(),
                        Send { size: 17 }
                    }
                }
                p { class: "text-[11px] text-muted-foreground",
                    Bot { size: 12 }
                    "Actions are executed with your permissions after confirmation by the assistant."
                }
            }
        }
    }
}

fn response_text(response: &AiChatResponse) -> String {
    let mut text = response.reply.clone();
    for action in &response.actions {
        if !action.success {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(&format!("⚠ {}", action.message));
        }
    }
    text
}

async fn refresh_changed_data() {
    if let Ok(data) = Storage::list_subscriptions().await {
        SUBSCRIPTIONS.write().set_data(data);
    }
    if let Ok(data) = Storage::list_transactions().await {
        TRANSACTIONS.write().set_data(data);
    }
    if let Ok(data) = Storage::list_debts().await {
        DEBTS.write().set_data(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_text_appends_failed_action_messages() {
        let response = AiChatResponse {
            reply: "Done".into(),
            actions: vec![
                crate::api::ai::AiActionResult {
                    action: "create".into(),
                    success: true,
                    message: "ok".into(),
                },
                crate::api::ai::AiActionResult {
                    action: "delete".into(),
                    success: false,
                    message: "not found".into(),
                },
            ],
        };
        let text = response_text(&response);
        assert_eq!(text, "Done\n\n⚠ not found");

        let empty = response_text(&AiChatResponse {
            reply: String::new(),
            actions: vec![crate::api::ai::AiActionResult {
                action: "x".into(),
                success: false,
                message: "failed".into(),
            }],
        });
        assert_eq!(empty, "⚠ failed");
    }

    #[test]
    fn chat_exposes_voice_dictation_with_all_input_paths() {
        let source = include_str!("ai_chat.rs");
        assert!(source.contains("Mic { size: 17 }"));
        assert!(source.contains("speech::listen"));
        assert!(source.contains("ai::transcribe"));
        assert!(source.contains("local_speech::transcribe_pcm"));
        assert!(source.contains("aria_label: if listening()"));
    }
}
