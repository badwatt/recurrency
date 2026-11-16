use dioxus::prelude::*;
use lucide_dioxus::{Bot, Send, Sparkles};

use crate::api::ai::{self, AiChatResponse, AiMessage};
use crate::components::dialog::Dialog;
use crate::components::toast::TOAST;
use crate::state::debts::DEBTS;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::state::transactions::TRANSACTIONS;
use crate::storage::Storage;

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
                    TOAST
                        .write()
                        .error(format!("AI assistant error: {error}"));
                }
            }
            sending.set(false);
        });
    });

    rsx! {
        Dialog {
            open: AI_CHAT_OPEN(),
            on_close: move |_| *AI_CHAT_OPEN.write() = false,
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
                    textarea {
                        value: input(),
                        oninput: move |event| input.set(event.value()),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter && !event.modifiers().shift() {
                                event.prevent_default();
                                send.call(());
                            }
                        },
                        rows: "2",
                        placeholder: "Ask for a subscription, transaction or debt…",
                        class: "min-h-11 flex-1 resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm",
                    }
                    button {
                        r#type: "submit",
                        disabled: sending(),
                        class: "inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        aria_label: "Send message",
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
