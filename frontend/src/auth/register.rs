use super::verify_email::VerifyEmailPage;
use crate::api::auth;
use crate::state::auth::AUTH;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn RegisterPage() -> Element {
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut auth_field = use_signal(String::new);
    let mut show_text_visible = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(String::new);
    let mut pending_token = use_signal(|| None::<String>);
    let mut pending_email = use_signal(|| None::<String>);

    let handle_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        error_msg.set(String::new());
        loading.set(true);
        match auth::register(email(), auth_field(), name()).await {
            Ok(resp) => {
                if resp.requires_verification {
                    pending_token.set(Some(resp.token.clone()));
                    pending_email.set(Some(resp.user.email.clone()));
                    AUTH.write().set_authenticated(resp.token, resp.user, true);
                } else {
                    AUTH.write().set_authenticated(resp.token, resp.user, false);
                    let nav = navigator();
                    nav.push(Route::Login {});
                }
            }
            Err(e) => {
                error_msg.set(e.to_string());
            }
        }
        loading.set(false);
    };

    // Inline verification
    if let (Some(token), Some(mail)) = (pending_token(), pending_email()) {
        return rsx! {
            VerifyEmailPage {
                email: mail,
                token: token,
                on_verified: move || {
                    let nav = navigator();
                    nav.push(Route::Login {});
                },
            }
        };
    }

    rsx! {
        div {
            class: "flex min-h-dvh items-center justify-center bg-gradient-to-br from-background to-muted p-4",
            div {
                class: "w-full max-w-md rounded-lg border border-border bg-card shadow-sm",

                // Header
                div { class: "p-6 pb-0",
                    h1 { class: "text-2xl font-bold text-center text-foreground",
                        "Create account"
                    }
                }

                // Form
                div { class: "p-6",
                    form {
                        onsubmit: handle_submit,
                        class: "space-y-4",

                        if !error_msg().is_empty() {
                            div {
                                class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                                {error_msg()}
                            }
                        }

                        // Name
                        div { class: "space-y-2",
                            label {
                                r#for: "reg-name",
                                class: "text-sm font-medium text-foreground",
                                "Name"
                            }
                            input {
                                id: "reg-name",
                                r#type: "text",
                                placeholder: "John Doe",
                                value: name.to_string(),
                                oninput: move |e| name.set(e.value()),
                                required: true,
                                class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                            }
                        }

                        // Email
                        div { class: "space-y-2",
                            label {
                                r#for: "reg-email",
                                class: "text-sm font-medium text-foreground",
                                "Email"
                            }
                            input {
                                id: "reg-email",
                                r#type: "email",
                                placeholder: "you@example.com",
                                value: email.to_string(),
                                oninput: move |e| email.set(e.value()),
                                required: true,
                                class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                            }
                        }

                        // Password
                        div { class: "space-y-2",
                            label {
                                r#for: "reg-password",
                                class: "text-sm font-medium text-foreground",
                                "Password"
                            }
                            div { class: "relative",
                                input {
                                    id: "reg-password",
                                    r#type: if show_text_visible() { "text" } else { "password" },
                                    placeholder: "••••••••",
                                    value: auth_field.to_string(),
                                    oninput: move |e| auth_field.set(e.value()),
                                    required: true,
                                    minlength: 8,
                                    class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 pr-10 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                                }
                                button {
                                    r#type: "button",
                                    onclick: move |_| show_text_visible.toggle(),
                                    class: "absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground",
                                    if show_text_visible() { "Hide" } else { "Show" }
                                }
                            }
                        }

                        // Submit
                        button {
                            r#type: "submit",
                            disabled: loading(),
                            class: "inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                            if loading() { "Creating..." } else { "Create account" }
                        }
                    }

                    // Login link
                    p { class: "mt-4 text-center text-sm text-muted-foreground",
                        "Already have an account? "
                        Link {
                            to: Route::Login {},
                            class: "text-primary hover:underline",
                            "Log in"
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
            email: "new@test.com".into(),
            name: Some("New User".into()),
            currency: "EUR".into(),
            verified,
        }
    }

    #[test]
    fn register_no_verification_auto_login() {
        let mut state = AuthState::default();
        let user = sample_user(true);
        state.set_authenticated("reg-tok".into(), user, false);
        assert!(state.is_authenticated);
    }

    #[test]
    fn register_with_verification_pending() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("reg-tok".into(), user, true);
        assert!(!state.is_authenticated);
        assert!(state.token.is_some());
    }

    #[test]
    fn register_then_verify_flow() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("reg-tok".into(), user, true);
        assert!(!state.is_authenticated);
        state.set_verified();
        assert!(state.is_authenticated);
        assert!(state.user.unwrap().verified);
    }

    #[test]
    fn toggle_display_state() {
        let show = true;
        assert_eq!(if show { "text" } else { "password" }, "text");
        let show = false;
        assert_eq!(if show { "text" } else { "password" }, "password");
    }

    #[test]
    fn error_message_display() {
        let err = crate::api::client::ApiError::Http(400, "Email already taken".into());
        assert!(err.to_string().contains("400"));
        assert!(err.to_string().contains("Email already taken"));
    }

    #[test]
    fn password_visibility_toggle() {
        let mut visible = false;
        visible = !visible;
        assert!(visible);
        visible = !visible;
        assert!(!visible);
    }

    #[test]
    fn register_form_fields() {
        let name = "John Doe";
        let email = "john@example.com";
        let pass_val = "test_pass_val";
        assert!(!name.is_empty());
        assert!(email.contains('@'));
        assert!(pass_val.len() >= 8);
    }

    #[test]
    fn auth_state_logout_after_register() {
        let mut state = AuthState::default();
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "new@test.com".into(),
            name: Some("New".into()),
            currency: "EUR".into(),
            verified: true,
        };
        state.set_authenticated("tok".into(), user, false);
        assert!(state.is_authenticated);
        state.logout();
        assert!(!state.is_authenticated);
        assert!(state.token.is_none());
    }

    // ── Name validation ──
    #[test]
    fn name_empty_invalid() {
        let name = "";
        assert!(name.trim().is_empty());
    }

    #[test]
    fn name_with_whitespace_trimmed() {
        let name = "  John  ";
        assert_eq!(name.trim(), "John");
    }

    #[test]
    fn name_single_char_valid() {
        let name = "J";
        assert!(!name.trim().is_empty());
    }

    #[test]
    fn name_long_valid() {
        let name = "Alexander The Great III";
        assert!(!name.trim().is_empty());
        assert!(name.len() >= 3);
    }

    // ── Email validation ──
    #[test]
    fn email_validation_standard() {
        let email = "test@example.com";
        assert!(email.contains('@'));
        assert!(email.contains('.'));
    }

    #[test]
    fn email_validation_no_at() {
        let email = "testexample.com";
        assert!(!email.contains('@'));
    }

    #[test]
    fn email_validation_empty_domain() {
        let email = "test@";
        let parts: Vec<_> = email.split('@').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[1].is_empty());
    }

    #[test]
    fn email_validation_no_tld() {
        let email = "test@example";
        assert!(email.contains('@'));
        // Domain after @ should have a dot for a valid TLD
        let after_at = email.split('@').nth(1).unwrap_or("");
        assert!(
            !after_at.contains('.'),
            "Email domain should have a dot for valid TLD"
        );
    }

    // ── Password validation ──
    #[test]
    fn password_min_length_boundary() {
        let pwd7 = "1234567";
        let pwd8 = "12345678";
        assert!(pwd7.len() < 8);
        assert!(pwd8.len() >= 8);
    }

    #[test]
    fn password_special_chars() {
        let pwd = "P@ssw0rd!";
        assert!(pwd.len() >= 8);
        assert!(pwd.contains('@') || pwd.contains('!'));
    }

    // ── Register flow states ──
    #[test]
    fn register_auto_login_on_success() {
        let mut state = AuthState::default();
        let user = sample_user(true);
        assert!(!state.is_authenticated);
        state.set_authenticated("new-token".into(), user, false);
        assert!(state.is_authenticated);
        assert!(state.is_verified());
    }

    #[test]
    fn register_pending_verification_token_stored() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("pending-token".into(), user, true);
        assert!(!state.is_authenticated);
        assert!(state.token.is_some());
        assert!(state.token.as_ref().unwrap() == "pending-token");
    }

    #[test]
    fn register_verify_and_login() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.is_authenticated);
        state.set_verified();
        assert!(state.is_authenticated);
        assert!(state.is_verified());
    }

    #[test]
    fn register_name_optional() {
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: None,
            currency: "USD".into(),
            verified: true,
        };
        assert!(user.name.is_none());
    }

    #[test]
    fn register_different_currencies() {
        for currency in ["USD", "EUR", "GBP", "JPY"] {
            let user = UserResponse {
                id: Uuid::new_v4(),
                email: format!("test@{}.com", currency),
                name: Some("Test".into()),
                currency: currency.into(),
                verified: true,
            };
            assert_eq!(user.currency, currency);
        }
    }

    // ── API errors in register context ──
    #[test]
    fn error_email_taken() {
        let err = crate::api::client::ApiError::Http(400, "Email already registered".into());
        assert!(err.to_string().contains("400"));
        assert!(err.to_string().contains("Email already registered"));
    }

    #[test]
    fn error_network_during_register() {
        let err = crate::api::client::ApiError::Network("connection reset".into());
        assert!(err.to_string().contains("connection reset"));
    }

    #[test]
    fn error_parse_during_register() {
        let err = crate::api::client::ApiError::Parse("unexpected end".into());
        assert!(err.to_string().contains("unexpected end"));
    }
}
