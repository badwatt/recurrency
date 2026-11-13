use crate::api::settings;
use crate::components::toast::TOAST;
use crate::layout::app_layout::{avatar_color, avatar_initials};
use crate::models::auth::{UpdateSettingsRequest, is_valid_email};
use crate::utils::image::base64_encode;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

const MAX_AVATAR_BYTES: u64 = 15_000_000;


/// Profile card: editable email and display name.
async fn resize_avatar(data_url: String) -> Option<String> {
    let mut eval = document::eval(r#"
        const source = await dioxus.recv();
        try {
            const image = new Image();
            const loaded = new Promise((resolve, reject) => {
                image.onload = resolve;
                image.onerror = reject;
                image.src = source;
            });
            await loaded;
            const max = 256;
            const scale = Math.min(1, max / image.width, max / image.height);
            const canvas = document.createElement('canvas');
            canvas.width = Math.max(1, Math.round(image.width * scale));
            canvas.height = Math.max(1, Math.round(image.height * scale));
            const context = canvas.getContext('2d');
            context.fillStyle = '#ffffff';
            context.fillRect(0, 0, canvas.width, canvas.height);
            context.drawImage(image, 0, 0, canvas.width, canvas.height);
            dioxus.send(canvas.toDataURL('image/jpeg', 0.86));
        } catch (_) {
            dioxus.send('');
        }
    "#);
    eval.send(data_url).ok()?;
    eval.recv::<String>()
        .await
        .ok()
        .filter(|value| value.starts_with("data:image/"))
}

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
        if !next_name.is_empty() && next_name.len() < 2 {
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
                        if cfg!(target_os = "android") {
                            button {
                                r#type: "button",
                                class: "inline-flex h-9 items-center justify-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent",
                                onclick: move |_| {
                                    spawn(async move {
                                        let mut eval = document::eval(r#"
                                            if (window.RecurrencyImagePicker) {
                                                await new Promise((resolve) => {
                                                    window.__recurrencyAvatarPicked = (data) => {
                                                        dioxus.send(data);
                                                        resolve();
                                                    };
                                                    window.RecurrencyImagePicker.pickImage();
                                                });
                                            } else {
                                                dioxus.send('');
                                            }
                                        "#);
                                        match eval.recv::<String>().await {
                                            Ok(data) if data.starts_with("data:image/") => {
                                                avatar.set(data);
                                                error.set(String::new());
                                            }
                                            Ok(_) => {}
                                            Err(_) => error.set(
                                                "Could not open the gallery. Try again.".to_string(),
                                            ),
                                        }
                                    });
                                },
                                "Change photo"
                            }
                        } else {
                            label {
                                class: "relative inline-flex h-9 cursor-pointer items-center justify-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent focus-within:ring-2 focus-within:ring-ring",
                                "Change photo"
                                input {
                                    id: "avatar-file",
                                    r#type: "file",
                                    accept: "image/*",
                                    aria_label: "Change profile photo",
                                    class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                                    onchange: move |e: Event<FormData>| {
                                        let Some(file) = e.files().first().cloned() else {
                                            return;
                                        };
                                        if file.size() > MAX_AVATAR_BYTES {
                                            error.set("Photo must be 15 MB or smaller".to_string());
                                            return;
                                        }
                                        let content_type = file
                                            .content_type()
                                            .unwrap_or_else(|| "image/jpeg".to_string());
                                        if !content_type.starts_with("image/") {
                                            error.set("Choose an image from your gallery".to_string());
                                            return;
                                        }
                                        error.set(String::new());
                                        spawn(async move {
                                            match file.read_bytes().await {
                                                Ok(bytes) => {
                                                    let original = format!(
                                                        "data:{content_type};base64,{}",
                                                        base64_encode(&bytes)
                                                    );
                                                    match resize_avatar(original.clone()).await {
                                                        Some(resized) => avatar.set(resized),
                                                        None if matches!(
                                                            content_type.as_str(),
                                                            "image/png" | "image/jpeg" | "image/webp"
                                                        ) && bytes.len() <= 300_000 =>
                                                        {
                                                            avatar.set(original);
                                                        }
                                                        None => error.set(
                                                            "Could not process that image. Try another photo."
                                                                .to_string(),
                                                        ),
                                                    }
                                                }
                                                Err(_) => error.set("Could not read the selected photo".to_string()),
                                            }
                                        });
                                    },
                                }
                            }
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
