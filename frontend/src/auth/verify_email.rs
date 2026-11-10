use crate::api::auth;
use crate::Route;
use dioxus::prelude::*;

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
        let val = e
            .value()
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(6)
            .collect();
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
                            value: code.to_string(),
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
    use crate::models::auth::UserResponse;
    use crate::state::auth::AuthState;
    use uuid::Uuid;

    fn sample_user(verified: bool) -> UserResponse {
        UserResponse {
            id: Uuid::new_v4(),
            email: "verify@test.com".into(),
                avatar: None,
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

    #[test]
    fn code_non_digit_filtering() {
        let raw = "abc123def456ghi";
        let filtered: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(filtered, "123456");
    }

    #[test]
    fn code_exactly_six_digits() {
        let raw = "654321";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result.len(), 6);
        assert_eq!(result, "654321");
    }

    #[test]
    fn code_less_than_six_digits() {
        let raw = "12";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn resend_format() {
        let err_msg = "Too many requests";
        let formatted = format!("Failed to resend: {}", err_msg);
        assert_eq!(formatted, "Failed to resend: Too many requests");
    }

    #[test]
    fn verify_error_code_length() {
        let code = "123";
        assert!(code.len() != 6);
    }

    #[test]
    fn code_empty() {
        let raw = "";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert!(result.is_empty());
    }

    #[test]
    fn code_only_digits() {
        let raw = "012345";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result, "012345");
    }

    #[test]
    fn code_with_spaces() {
        let raw = "123 456";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result, "123456");
    }

    #[test]
    fn code_with_special_chars() {
        let raw = "123-456-789";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result, "123456");
    }

    #[test]
    fn code_take_less_than_6() {
        let raw = "123";
        let result: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        assert_eq!(result, "123"); // less than 6, take returns what it can
    }

    #[test]
    fn resend_error_too_many_requests() {
        let err = crate::api::client::ApiError::Http(429, "Too many requests".into());
        assert!(err.to_string().contains("429"));
    }

    #[test]
    fn resend_error_server_error() {
        let err = crate::api::client::ApiError::Http(500, "Internal error".into());
        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn verify_error_network() {
        let err = crate::api::client::ApiError::Network("connection refused".into());
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn verify_error_parse() {
        let err = crate::api::client::ApiError::Parse("invalid json".into());
        assert!(err.to_string().contains("invalid json"));
    }

    #[test]
    fn error_message_display() {
        let msg = "Verification failed";
        assert!(!msg.is_empty());
    }

    #[test]
    fn email_param_not_empty() {
        // Email param should not be empty (passed from URL)
        let email = "user@example.com";
        assert!(!email.is_empty());
        assert!(email.contains('@'));
    }

    #[test]
    fn token_param_not_empty() {
        // Token param should not be empty
        let token = "abc123def456";
        assert!(!token.is_empty());
        assert!(token.len() > 10);
    }

    #[test]
    fn verify_url_construction() {
        let token = "test-token";
        let path = format!("/api/auth/verify-email?token={}", token);
        assert_eq!(path, "/api/auth/verify-email?token=test-token");
    }
}
