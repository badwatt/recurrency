use dioxus::prelude::*;
use crate::api::auth;
use crate::Route;

#[component]
pub fn VerifyEmailPage(email: String, token: String, on_verified: EventHandler<()>) -> Element {
    let mut code = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut resending = use_signal(|| false);
    let mut error_msg = use_signal(String::new);
    // Store props in signals so closures can read (copy) them without moving
    let token_sig = use_signal(|| token);
    let on_verified_sig = use_signal(|| on_verified);

    let handle_verify = move |evt: FormEvent| {
        evt.prevent_default();
        let c = code();
        if c.len() != 6 {
            error_msg.set("Please enter a 6-digit code".into());
            return;
        }
        error_msg.set(String::new());
        loading.set(true);
        let tok = token_sig();
        let ov = on_verified_sig();
        spawn(async move {
            match auth::verify_email(&tok, &c).await {
                Ok(_) => {
                    ov.call(());
                }
                Err(e) => {
                    error_msg.set(e.to_string());
                }
            }
            loading.set(false);
        });
    };

    let handle_resend = move |_evt: MouseEvent| {
        resending.set(true);
        let tok = token_sig();
        spawn(async move {
            match auth::resend_verification(&tok).await {
                Ok(_) => {}
                Err(e) => {
                    error_msg.set(format!("Failed to resend: {}", e));
                }
            }
            resending.set(false);
        });
    };

    let on_code_input = move |e: FormEvent| {
        let val = e.value().chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        code.set(val);
    };

    rsx! {
        div {
            class: "flex min-h-dvh items-center justify-center bg-gradient-to-br from-background to-muted p-4",
            div {
                class: "w-full max-w-md rounded-lg border border-border bg-card shadow-sm",

                // Header
                div { class: "p-6 pb-2 text-center",
                    // Mail icon
                    div {
                        class: "mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10",
                        span {
                            class: "text-lg text-primary",
                            "✉"
                        }
                    }
                    h1 { class: "text-2xl font-bold text-foreground",
                        "Check your email"
                    }
                    p { class: "mt-2 text-sm text-muted-foreground",
                        "We sent a 6-digit code to"
                        br {}
                        span { class: "font-medium text-foreground", {email.clone()} }
                    }
                }

                // Form
                div { class: "p-6 pt-2",
                    form {
                        onsubmit: handle_verify,
                        class: "space-y-4",

                        if !error_msg().is_empty() {
                            div {
                                class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                                {error_msg()}
                            }
                        }

                        input {
                            r#type: "text",
                            placeholder: "Enter 6-digit code",
                            value: "{code}",
                            oninput: on_code_input,
                            maxlength: 6,
                            autofocus: true,
                            class: "flex h-12 w-full rounded-md border border-border bg-background px-3 py-2 text-center text-2xl tracking-widest font-mono text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                        }

                        button {
                            r#type: "submit",
                            disabled: loading() || code().len() != 6,
                            class: "inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                            if loading() { "Verifying..." } else { "Verify Email" }
                        }
                    }

                    // Footer: back + resend
                    div { class: "mt-4 flex items-center justify-between text-sm",
                        Link {
                            to: Route::Login {},
                            class: "flex items-center text-muted-foreground hover:text-foreground",
                            "← Back to login"
                        }
                        button {
                            r#type: "button",
                            disabled: resending(),
                            onclick: handle_resend,
                            class: "text-sm text-muted-foreground hover:text-foreground disabled:opacity-50",
                            if resending() { "Sending..." } else { "Resend code" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::auth::AuthState;
    use crate::models::auth::UserResponse;
    use uuid::Uuid;

    fn sample_user(verified: bool) -> UserResponse {
        UserResponse {
            id: Uuid::new_v4(),
            email: "verify@test.com".into(),
            name: Some("Verify User".into()),
            currency: "EUR".into(),
            verified,
        }
    }

    #[test]
    fn verify_email_sets_authenticated() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("v-tok".into(), user, true);
        assert!(!state.is_authenticated);
        state.set_verified();
        assert!(state.is_authenticated);
        assert!(state.user.unwrap().verified);
    }

    #[test]
    fn code_filter_digits() {
        let raw = "1a2b3c4d5e6f";
        let filtered: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(filtered, "123456");
    }

    #[test]
    fn code_length_validation() {
        let code = "1234";
        assert_ne!(code.len(), 6, "Code must be 6 digits");
        let code = "123456";
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn resend_error_display() {
        let err = crate::api::client::ApiError::Http(429, "Too many requests".into());
        let msg = format!("Failed to resend: {}", err);
        assert!(msg.contains("429"));
    }

    #[test]
    fn verify_state_after_register() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.is_authenticated);
        assert!(state.token.is_some());
        state.set_verified();
        assert!(state.is_authenticated);
    }

    #[test]
    fn code_truncation() {
        let raw = "1234567890";
        let truncated: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(truncated, "123456");
    }
}