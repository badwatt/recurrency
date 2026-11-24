//! In-app BYOK assistant. The conversation is local to the open dialog.

use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use base64::Engine;
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
#[cfg(not(target_arch = "wasm32"))]
use nsh_core::file_dialog::{FileFilter, OpenDialog};
use nsh_core::prelude::*;

use crate::api::{self, AiMessage};
use crate::auth::{self, AuthSession};
use crate::components::button::{ButtonProps, button as shared_button};

const WELCOME: &str =
    "Hi, I can create, update or delete subscriptions, transactions and debts. What do you need?";

#[derive(Clone, Debug, PartialEq)]
struct ChatLine {
    role: String,
    content: String,
}

pub fn ai_chat(auth: AuthSession) -> AnyElement {
    let messages = signal(vec![ChatLine {
        role: "assistant".to_string(),
        content: WELCOME.to_string(),
    }]);
    let draft = signal(String::new());
    let sending = signal(false);
    let error = signal(String::new());
    let send: Rc<dyn Fn()> = {
        let token = auth.token.clone();
        let send_messages = messages;
        let send_draft = draft;
        let send_sending = sending;
        let send_error = error;
        Rc::new(move || {
            let text = send_draft.get().trim().to_string();
            if text.is_empty() || send_sending.get() {
                return;
            }
            send_draft.set(String::new());
            send_error.set(String::new());
            send_messages.set({
                let mut next = send_messages.get();
                next.push(ChatLine {
                    role: "user".to_string(),
                    content: text,
                });
                next
            });
            send_sending.set(true);
            let server = auth::stored_server();
            let token = token.clone();
            spawn(async move {
                let history: Vec<AiMessage> = send_messages
                    .get()
                    .into_iter()
                    .map(|line| AiMessage {
                        role: line.role,
                        content: line.content,
                    })
                    .collect();
                match api::ai_chat(server, token, &history).await {
                    Ok(response) => {
                        let content = if response.actions.is_empty() {
                            response.reply
                        } else {
                            format!(
                                "{}\n\n{} action(s) applied.",
                                response.reply,
                                response.actions.len()
                            )
                        };
                        let mut next = send_messages.get();
                        next.push(ChatLine {
                            role: "assistant".to_string(),
                            content,
                        });
                        send_messages.set(next);
                    }
                    Err(message) => send_error.set(message),
                }
                send_sending.set(false);
            });
        })
    };
    let transcribing = signal(false);
    let recording = signal(false);
    let voice = {
        let token = auth.token.clone();
        move || {
            if transcribing.get() {
                return;
            }
            let token = token.clone();
            error.set(String::new());
            transcribing.set(true);
            spawn(async move {
                match pick_audio().await {
                    Ok(Some((mime, audio))) => {
                        match api::transcribe(auth::stored_server(), token, &audio, &mime).await {
                            Ok(text) => {
                                let mut next = draft.get();
                                if !next.trim().is_empty() {
                                    next.push(' ');
                                }
                                next.push_str(text.trim());
                                draft.set(next);
                            }
                            Err(message) => error.set(message),
                        }
                    }
                    Ok(None) => {}
                    Err(problem) => error.set(problem),
                }
                transcribing.set(false);
            });
        }
    };
    let record = {
        let token = auth.token.clone();
        move || {
            if transcribing.get() {
                return;
            }
            let token = token.clone();
            error.set(String::new());
            spawn(async move {
                match crate::mic::toggle().await {
                    Ok(crate::mic::Capture::Started) => recording.set(true),
                    Ok(crate::mic::Capture::Stopped { base64, mime }) => {
                        recording.set(false);
                        transcribing.set(true);
                        match api::transcribe(
                            auth::stored_server(),
                            token,
                            &base64,
                            if mime.is_empty() { "audio/webm" } else { &mime },
                        )
                        .await
                        {
                            Ok(text) => {
                                let mut next = draft.get();
                                if !next.trim().is_empty() {
                                    next.push(' ');
                                }
                                next.push_str(text.trim());
                                draft.set(next);
                            }
                            Err(message) => error.set(message),
                        }
                        transcribing.set(false);
                    }
                    Err(message) => {
                        recording.set(false);
                        error.set(message);
                    }
                }
            });
        }
    };
    let voice_press: Rc<dyn Fn()> = if crate::mic::supported() {
        Rc::new(record)
    } else {
        Rc::new(voice)
    };
    let submit = send.clone();
    let press = send.clone();
    let lines = move || {
        messages
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let user = line.role == "user";
                view()
                    .key(index.to_string())
                    .max_w(px(if user { 380.0 } else { 430.0 }))
                    .p(px(10.0))
                    .rounded(px(10.0))
                    .bg(move || {
                        if user {
                            theme().primary
                        } else {
                            theme().muted.opacity(0.5)
                        }
                    })
                    .child(text(line.content).text_size(px(12.0)).text_color(move || {
                        if user {
                            theme().primary_foreground
                        } else {
                            theme().foreground
                        }
                    }))
                    .into_element()
            })
            .collect::<Vec<_>>()
    };
    let close = move || crate::ui::close_ai_chat();
    view()
        .absolute()
        .inset_0()
        .z_index(100)
        .items_center()
        .justify_center()
        .p(px(16.0))
        .bg(rgba(0x000000cc))
        .child(
            view()
                .w_full()
                .max_w(px(560.0))
                .h(px(620.0))
                .max_h(px(620.0))
                .flex_col()
                .rounded(px(14.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .overflow_hidden()
                .child(
                    view()
                        .w_full()
                        .p(px(18.0))
                        .pb(px(12.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    view()
                                        .flex_col()
                                        .child(
                                            text("AI assistant")
                                                .text_size(px(16.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().foreground),
                                        )
                                        .child(
                                            text(
                                                "Create, update or delete items with your own key.",
                                            )
                                            .text_size(px(11.0))
                                            .text_color(move || theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    button()
                                        .cursor_pointer()
                                        .items_center()
                                        .justify_center()
                                        .h(px(28.0))
                                        .w(px(28.0))
                                        .rounded(px(5.0))
                                        .on_press(close)
                                        .child(
                                            image(asset!("icons/x.svg"))
                                                .size(px(14.0))
                                                .tint_color(move || theme().muted_foreground),
                                        ),
                                ),
                        ),
                )
                .child(
                    scroll()
                        .flex_1()
                        .p(px(16.0))
                        .child(view().w_full().flex_col().gap(px(8.0)).children(lines())),
                )
                .child(
                    view()
                        .w_full()
                        .p(px(14.0))
                        .border_t(px(1.0))
                        .border_color(move || theme().border)
                        .gap(px(8.0))
                        .child(
                            view()
                                .flex_row()
                                .items_end()
                                .gap(px(8.0))
                                .child(
                                    input()
                                        .value(draft)
                                        .on_change(move |value: &str| draft.set(value.to_string()))
                                        .on_submit(move |_: &str| submit())
                                        .placeholder("Ask for a subscription, transaction or debt…")
                                        .auto_fill(AutoFill::Off)
                                        .flex_grow(1.0)
                                        .h(px(38.0))
                                        .px(px(10.0))
                                        .rounded(px(7.0))
                                        .border(px(1.0))
                                        .border_color(move || theme().input)
                                        .bg(move || theme().background)
                                        .text_size(px(13.0))
                                        .text_color(move || theme().foreground),
                                )
                                .child(shared_button(
                                    ButtonProps::new(if transcribing.get() {
                                        "Transcribing…"
                                    } else if recording.get() {
                                        "Stop recording"
                                    } else if crate::mic::supported() {
                                        "Record"
                                    } else {
                                        "Voice note"
                                    })
                                    .variant(crate::components::button::ButtonVariant::Outline)
                                    .icon(asset!("icons/mic.svg"))
                                    .disabled(transcribing.get())
                                    .on_press(move || voice_press()),
                                ))
                                .child(shared_button(
                                    ButtonProps::new(if sending.get() {
                                        "Thinking…"
                                    } else {
                                        "Send"
                                    })
                                    .disabled(sending.get())
                                    .on_press(move || press()),
                                )),
                        )
                        .child(
                            text(move || error.get())
                                .text_size(px(11.0))
                                .text_color(move || theme().destructive),
                        ),
                ),
        )
        .into_element()
}

async fn pick_audio() -> Result<Option<(String, String)>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(url) = crate::web_files::pick_data_url("audio/*,video/webm").await? else {
            return Ok(None);
        };
        let (meta, encoded) = url
            .split_once(',')
            .ok_or_else(|| "Could not read the audio file.".to_string())?;
        let mime = meta
            .strip_prefix("data:")
            .and_then(|value| value.split(';').next())
            .filter(|value| !value.is_empty())
            .unwrap_or("audio/webm")
            .to_string();
        Ok(Some((mime, encoded.to_string())))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match pick_paths(OpenDialog::files().title("Transcribe a voice note").filter(
            FileFilter::new("Audio", ["wav", "mp3", "m4a", "ogg", "webm", "mp4"]),
        ))
        .await?
        {
            Some(paths) => match paths.first() {
                Some(path) => {
                    let bytes = std::fs::read(path)
                        .map_err(|problem| format!("Could not read the audio file: {problem}"))?;
                    let audio = base64::engine::general_purpose::STANDARD.encode(bytes);
                    Ok(Some((audio_mime(path).to_string(), audio)))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn audio_mime(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("m4a") | Some("mp4") => "audio/mp4",
        Some("ogg") => "audio/ogg",
        Some("webm") => "audio/webm",
        _ => "audio/wav",
    }
}

#[cfg(test)]
mod tests {
    use super::audio_mime;

    #[test]
    fn audio_mime_follows_the_file_extension() {
        assert_eq!(audio_mime(std::path::Path::new("note.mp3")), "audio/mpeg");
        assert_eq!(audio_mime(std::path::Path::new("note.ogg")), "audio/ogg");
        assert_eq!(
            audio_mime(std::path::Path::new("note.unknown")),
            "audio/wav"
        );
    }
}
