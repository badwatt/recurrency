use crate::api::settings;
use crate::components::toast::TOAST;
use crate::models::auth::UpdateSettingsRequest;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

/// Profile card: read-only email and editable display name.
#[component]
pub fn ProfileCard() -> Element {
    let mut name = use_signal(|| {
        AUTH()
            .user
            .as_ref()
            .and_then(|u| u.name.clone())
            .unwrap_or_default()
    });
    let mut saving_name = use_signal(|| false);

    // Sync name if it is empty (never overwrite the user's in-progress edit).
    use_effect(move || {
        if let Some(u) = &AUTH().user
            && name().is_empty()
        {
            name.set(u.name.clone().unwrap_or_default());
        }
    });

    let user_email = AUTH()
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Personal information" }
                p { class: "text-sm text-muted-foreground", "Change the name we use to greet you" }
            }
            div { class: "p-6 space-y-4",
                // Email (read-only)
                div { class: "space-y-2",
                    label { r#for: "settings-email", class: "text-sm font-medium", "Email" }
                    input {
                        id: "settings-email",
                        r#type: "email",
                        value: user_email.to_string(),
                        disabled: true,
                        class: "flex h-10 w-full rounded-md border border-input bg-muted px-3 py-2 text-sm opacity-50 cursor-not-allowed",
                    }
                }
                // Name
                div { class: "space-y-2",
                    label { r#for: "settings-name", class: "text-sm font-medium", "Name" }
                    input {
                        id: "settings-name",
                        r#type: "text",
                        value: name.to_string(),
                        oninput: move |e| name.set(e.value()),
                        onblur: move |_| {
                            let n = name();
                            if n.len() >= 2 {
                                saving_name.set(true);
                                spawn(async move {
                                    match settings::update(&UpdateSettingsRequest {
                                        name: Some(n),
                                        currency: None,
                                    }).await {
                                        Ok(resp) => {
                                            AUTH.write().user = Some(resp.user);
                                            AUTH.write().persist_to_storage();
                                            TOAST.write().success("Name updated");
                                            saving_name.set(false);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to save name: {}", e);
                                            TOAST.write().error("Failed to update name");
                                            saving_name.set(false);
                                        }
                                    }
                                });
                            }
                        },
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                    }
                }

            }
        }
    }
}
