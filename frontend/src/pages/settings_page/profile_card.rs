use crate::api::settings;
use crate::components::toast::TOAST;
use crate::layout::app_layout::{avatar_color, avatar_initials};
use crate::models::auth::{UpdateSettingsRequest, is_valid_email};
use crate::state::auth::AUTH;
use dioxus::prelude::*;

const MAX_AVATAR_BYTES: u64 = 200_000;

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[((a & 0b11) << 4 | b >> 4) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((b & 0b1111) << 2 | c >> 6) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(c & 0b111111) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Profile card: editable email and display name.
#[component]
pub fn ProfileCard() -> Element {
    let mut email = use_signal(|| {
        AUTH()
            .user
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_default()
    });
    let mut name = use_signal(|| {
        AUTH()
            .user
            .as_ref()
            .and_then(|u| u.name.clone())
            .unwrap_or_default()
    });
    let mut avatar = use_signal(|| {
        AUTH()
            .user
            .as_ref()
            .and_then(|u| u.avatar.clone())
            .unwrap_or_default()
    });
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);

    let save = move |_| {
        let next_email = email().trim().to_lowercase();
        let next_name = name().trim().to_string();

        if !is_valid_email(&next_email) {
            error.set("Enter a valid email address".to_string());
            return;
        }
        if next_name.len() < 2 {
            error.set("Name must be at least 2 characters".to_string());
            return;
        }

        error.set(String::new());
        saving.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                email: Some(next_email),
                avatar: Some(avatar()),
                name: Some(next_name),
                currency: None,
            })
            .await
            {
                Ok(resp) => {
                    email.set(resp.user.email.clone());
                    name.set(resp.user.name.clone().unwrap_or_default());
                    avatar.set(resp.user.avatar.clone().unwrap_or_default());
                    AUTH.write().user = Some(resp.user);
                    AUTH.write().persist_to_storage();
                    TOAST.write().success("Profile updated");
                }
                Err(e) => {
                    log::error!("Failed to save profile: {}", e);
                    error.set(e.to_string());
                }
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Personal information" }
                p { class: "text-sm text-muted-foreground", "Your email identifies you; your name is how we greet you" }
            }
            div { class: "p-6 flex flex-col gap-4",
                div { class: "flex items-center gap-4",
                    if avatar().is_empty() {
                        {let name = AUTH().user.as_ref().and_then(|u| u.name.clone());
                         let email = AUTH().user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
                         let initials = avatar_initials(name.as_deref(), &email);
                         let color = avatar_color(&email);
                         rsx! {
                            span {
                                class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-full text-lg font-semibold text-white",
                                style: "background-color: {color}",
                                {initials}
                            }
                         }}
                    } else {
                        img {
                            src: avatar(),
                            alt: "Profile picture",
                            class: "h-16 w-16 shrink-0 rounded-full object-cover",
                        }
                    }
                    div { class: "flex flex-col gap-2",
                        label {
                            r#for: "avatar-file",
                            class: "inline-flex h-9 cursor-pointer items-center justify-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent",
                            "Change photo"
                        }
                        input {
                            id: "avatar-file",
                            r#type: "file",
                            accept: "image/png,image/jpeg,image/webp",
                            class: "sr-only",
                            onchange: move |e: Event<FormData>| {
                                let Some(file) = e.files().first().cloned() else {
                                    return;
                                };
                                if file.size() > MAX_AVATAR_BYTES {
                                    error.set("Photo must be 200 KB or smaller".to_string());
                                    return;
                                }
                                let Some(content_type) = file.content_type() else {
                                    error.set("Could not read the selected photo".to_string());
                                    return;
                                };
                                if !matches!(
                                    content_type.as_str(),
                                    "image/png" | "image/jpeg" | "image/webp"
                                ) {
                                    error.set("Use a PNG, JPEG or WebP photo".to_string());
                                    return;
                                }
                                error.set(String::new());
                                spawn(async move {
                                    match file.read_bytes().await {
                                        Ok(bytes) => {
                                            avatar.set(format!(
                                                "data:{content_type};base64,{}",
                                                base64_encode(&bytes)
                                            ));
                                        }
                                        Err(_) => error.set("Could not read the selected photo".to_string()),
                                    }
                                });
                            },
                        }
                        if !avatar().is_empty() {
                            button {
                                r#type: "button",
                                class: "text-left text-xs text-muted-foreground hover:text-destructive",
                                onclick: move |_| avatar.set(String::new()),
                                "Remove photo"
                            }
                        }
                    }
                }
                div { class: "space-y-2",
                    label { r#for: "settings-email", class: "text-sm font-medium", "Email" }
                    input {
                        id: "settings-email",
                        r#type: "email",
                        autocomplete: "email",
                        value: email.to_string(),
                        oninput: move |e| email.set(e.value()),
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                    }
                }
                div { class: "space-y-2",
                    label { r#for: "settings-name", class: "text-sm font-medium", "Name" }
                    input {
                        id: "settings-name",
                        r#type: "text",
                        autocomplete: "name",
                        value: name.to_string(),
                        oninput: move |e| name.set(e.value()),
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                    }
                }
                if !error().is_empty() {
                    p { class: "text-sm text-destructive", role: "alert", {error()} }
                }
                div { class: "flex justify-end",
                    button {
                        r#type: "button",
                        class: "inline-flex h-10 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: saving(),
                        onclick: save,
                        if saving() { "Saving..." } else { "Save changes" }
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
    fn base64_encodes_known_values() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_pads_remaining_bytes() {
        assert!(base64_encode(&[0]).ends_with("=="));
        assert!(base64_encode(&[0, 0]).ends_with("="));
        assert!(!base64_encode(&[0, 0, 0]).ends_with('='));
    }
}
