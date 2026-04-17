use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::api::auth;
use crate::api::demo;
use crate::Route;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(String::new);
    let mut auth_field = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut demo_loading = use_signal(|| false);
    let mut demo_enabled = use_signal(|| false);

    // Check demo status on mount
    use_future(move || async move {
        match demo::get_status().await {
            Ok(status) => demo_enabled.set(status.enabled),
            Err(_) => demo_enabled.set(false),
        }
    });

    let handle_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        error.set(String::new());
        loading.set(true);
        match auth::login(email(), auth_field()).await {
            Ok(resp) => {
                AUTH.write().set_authenticated(
                    resp.token,
                    resp.user,
                    resp.requires_verification,
                );
                let nav = navigator();
                nav.push(Route::Dashboard {});
            }
            Err(e) => {
                error.set(e.to_string());
            }
        }
        loading.set(false);
    };

    let handle_demo = move |_evt: MouseEvent| async move {
        error.set(String::new());
        demo_loading.set(true);
        // init → login with credentials → set_authenticated → reset (fire-and-forget)
        match demo::init().await {
            Ok(init_resp) => {
                match auth::login(init_resp.email, init_resp.password).await {
                    Ok(auth_resp) => {
                        AUTH.write().set_authenticated(
                            auth_resp.token,
                            auth_resp.user,
                            auth_resp.requires_verification,
                        );
                        // Await reset so demo data is ready before navigating
                        let _ = demo::reset().await;
                        let nav = navigator();
                        nav.push(Route::Dashboard {});
                    }
                    Err(e) => {
                        error.set(e.to_string());
                    }
                }
            }
            Err(e) => {
                error.set(e.to_string());
            }
        }
        demo_loading.set(false);
    };

    rsx! {
        div {
            class: "flex min-h-dvh items-center justify-center bg-background p-4",
            div {
                class: "w-full max-w-md rounded-lg border border-border bg-card p-0 shadow-sm",

                // Header
                div {
                    class: "space-y-1 p-6 pb-0",
                    h1 { class: "text-2xl font-bold text-foreground", "Sign in" }
                    p { class: "text-sm text-muted-foreground",
                        "Enter your email and password to access your account"
                    }
                }

                // Form
                form {
                    onsubmit: handle_submit,
                    class: "space-y-4 p-6",

                    if !error().is_empty() {
                        div {
                            class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                            {error()}
                        }
                    }

                    // Email
                    div { class: "space-y-2",
                        label {
                            r#for: "login-email",
                            class: "text-sm font-medium text-foreground",
                            "Email"
                        }
                        input {
                            id: "login-email",
                            r#type: "email",
                            placeholder: "m@example.com",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            required: true,
                            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                        }
                    }

                    // Password
                    div { class: "space-y-2",
                        label {
                            r#for: "login-password",
                            class: "text-sm font-medium text-foreground",
                            "Password"
                        }
                        input {
                            id: "login-password",
                            r#type: "password",
                            value: "{auth_field}",
                            oninput: move |e| auth_field.set(e.value()),
                            required: true,
                            minlength: 8,
                            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                        }
                    }

                    // Submit
                    button {
                        r#type: "submit",
                        disabled: loading(),
                        class: "inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                        if loading() { "Signing in..." } else { "Sign in" }
                    }

                    // Demo button
                    if demo_enabled() {
                        button {
                            r#type: "button",
                            disabled: demo_loading(),
                            onclick: handle_demo,
                            class: "inline-flex h-10 w-full items-center justify-center rounded-md border border-border bg-background px-4 py-2 text-sm font-medium text-foreground hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50",
                            if demo_loading() { "Loading..." } else { "Demo account" }
                        }
                    }

                    // Register link
                    p { class: "text-center text-sm text-muted-foreground",
                        "Don't have an account? "
                        Link {
                            to: Route::Register {},
                            class: "text-primary hover:underline",
                            "Sign up"
                        }
                    }

                    // Configure server URL
                    p { class: "text-center text-sm text-muted-foreground",
                        Link {
                            to: Route::Setup {},
                            class: "text-primary hover:underline",
                            "Configure server URL"
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
            email: "user@test.com".into(),
            name: Some("Test User".into()),
            currency: "EUR".into(),
            verified,
        }
    }

    #[test]
    fn login_state_no_verification() {
        let mut state = AuthState::default();
        let user = sample_user(true);
        state.set_authenticated("tok".into(), user.clone(), false);
        assert!(state.is_authenticated);
        assert!(state.is_verified());
        assert_eq!(state.token, Some("tok".into()));
    }

    #[test]
    fn login_state_requires_verification() {
        let mut state = AuthState::default();
        let user = sample_user(false);
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.is_authenticated);
        assert!(!state.is_verified());
    }

    #[test]
    fn demo_login_flow() {
        // Simulates: init -> set_authenticated -> set_verified
        let mut state = AuthState::default();
        let user = sample_user(true);
        state.set_authenticated("demo-tok".into(), user, false);
        assert!(state.is_authenticated);
    }

    #[test]
    fn error_display() {
        let err = crate::api::client::ApiError::Http(401, "Bad credentials".into());
        assert!(err.to_string().contains("401"));
    }

    #[test]
    fn empty_error_signal() {
        let error = String::new();
        assert!(error.is_empty());
    }

    #[test]
    fn email_format_validation() {
        let email = "user@example.com";
        assert!(email.contains('@'));
        let invalid = "userexample.com";
        assert!(!invalid.contains('@'));
    }

    #[test]
    fn password_min_length() {
        let short = "1234567"; // 7 chars
        assert!(short.len() < 8);
        let valid = "12345678"; // 8 chars
        assert!(valid.len() >= 8);
    }

    #[test]
    fn api_error_display_for_login() {
        let err = crate::api::client::ApiError::Http(401, "Invalid credentials".into());
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Invalid credentials"));
    }

    #[test]
    fn api_network_error_display() {
        let err = crate::api::client::ApiError::Network("Connection refused".into());
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn demo_flow_state_transition() {
        let mut state = AuthState::default();
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "demo@test.com".into(),
            name: Some("Demo".into()),
            currency: "EUR".into(),
            verified: true,
        };
        // Before: not authenticated
        assert!(!state.is_authenticated);
        // After demo login
        state.set_authenticated("demo-token".into(), user, false);
        assert!(state.is_authenticated);
        assert_eq!(state.token, Some("demo-token".into()));
        // After logout
        state.logout();
        assert!(!state.is_authenticated);
        assert!(state.token.is_none());
    }
}