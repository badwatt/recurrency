use crate::api::settings;
use crate::components::toast::TOAST;
use crate::layout::app_layout::{avatar_color, avatar_initials};
use crate::models::auth::{UpdateSettingsRequest, is_valid_email};
use crate::state::auth::AUTH;
use crate::utils::image::base64_encode;
use dioxus::prelude::*;

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::SectionCard;
use crate::components::field::Field;
use crate::components::input::Input;
const MAX_AVATAR_BYTES: u64 = 15_000_000;

/// Profile card: editable email and display name.
async fn resize_avatar(data_url: String) -> Option<String> {
    let mut eval = document::eval(
        r#"
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
    "#,
    );
    eval.send(data_url).ok()?;
    eval.recv::<String>()
        .await
        .ok()
        .filter(|value| value.starts_with("data:image/"))
}

/// Read and downscale a desktop image without relying on WebView canvas APIs.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
))]
async fn pick_desktop_avatar() -> Result<String, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
        .await
    else {
        return Err("No photo selected".to_string());
    };
    let bytes = file.read().await;
    if bytes.len() as u64 > MAX_AVATAR_BYTES {
        return Err("Photo must be 15 MB or smaller".to_string());
    }
    let image = image::load_from_memory(&bytes)
        .map_err(|_| "Could not process that image. Try a PNG or JPEG.".to_string())?;
    let resized = image
        .resize(256, 256, image::imageops::FilterType::Lanczos3)
        .to_rgb8();
    let mut output = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 86)
        .encode_image(&resized)
        .map_err(|_| "Could not process that image. Try a PNG or JPEG.".to_string())?;
    Ok(format!("data:image/jpeg;base64,{}", base64_encode(&output)))
}

#[cfg(not(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
)))]
async fn pick_desktop_avatar() -> Result<String, String> {
    Err("The native photo picker is unavailable".to_string())
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
        SectionCard {
            title: "Personal information",
            description: "Your email identifies you; your name is how we greet you",
            content_class: "p-0",
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
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
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
                        } else if cfg!(all(not(target_arch = "wasm32"), not(target_os = "android"))) {
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    spawn(async move {
                                        match pick_desktop_avatar().await {
                                            Ok(data) => {
                                                avatar.set(data);
                                                error.set(String::new());
                                            }
                                            Err(message) => error.set(message),
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
                                    name: "avatar",
                                    autocomplete: "off",
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
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Xs,
                                class: "text-muted-foreground hover:text-destructive",
                                onclick: move |_| avatar.set(String::new()),
                                "Remove photo"
                            }
                        }
                    }
                }
                Field { label: "Email", html_for: "settings-email",
                    Input {
                        id: "settings-email",
                        r#type: "email",
                        autocomplete: "email",
                        value: email.to_string(),
                        on_input: move |value| email.set(value),
                    }
                }
                Field { label: "Name", html_for: "settings-name",
                    Input {
                        id: "settings-name",
                        r#type: "text",
                        autocomplete: "name",
                        value: name.to_string(),
                        on_input: move |value| name.set(value),
                    }
                }
                if !error().is_empty() {
                    p { class: "text-sm text-destructive", role: "alert", {error()} }
                }
                div { class: "flex justify-end",
                    Button {
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

    #[test]
    fn desktop_avatar_uses_native_picker_and_resizes() {
        let src = include_str!("profile_card.rs");
        assert!(src.contains("AsyncFileDialog"));
        assert!(src.contains("image::load_from_memory"));
        assert!(src.contains("pick_desktop_avatar"));
    }
}
