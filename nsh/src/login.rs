//! Native sign-in surface, matching the Dioxus card layout.

use crate::auth::{self, Session};
use crate::components::button::{ButtonProps, button as shared_button};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

const CARD_WIDTH: f32 = 448.0;

pub fn login_screen(session: Signal<Session>) -> impl IntoElement {
    let registering = signal(false);
    let name = signal(String::new());
    let email = signal(String::new());
    let password = signal(String::new());
    let server = signal(auth::stored_server());
    let edit_server = signal(false);
    let error = signal(String::new());
    let busy = signal(false);
    let demo_enabled = signal(false);
    let demo_loading = signal(false);
    let verification = signal(None::<(String, String)>);
    let demo_server = auth::stored_server();
    spawn(async move {
        if auth::demo_enabled(&demo_server).await {
            demo_enabled.set(true);
        }
    });
    let demo = {
        let server = server;
        move || {
            if demo_loading.get() {
                return;
            }
            demo_loading.set(true);
            error.set(String::new());
            let server_value = server.get().trim().trim_end_matches('/').to_string();
            spawn(async move {
                match auth::demo_login(&server_value).await {
                    Ok(next) => {
                        demo_loading.set(false);
                        session.set(Session::SignedIn(next));
                        return;
                    }
                    Err(message) => error.set(message),
                }
                demo_loading.set(false);
            });
        }
    };

    let submit = move || {
        if busy.get() {
            return;
        }
        let email_value = email.get().trim().to_string();
        let password_value = password.get();
        let name_value = name.get().trim().to_string();
        if email_value.is_empty() || password_value.is_empty() {
            error.set("Enter your email and password".to_string());
            return;
        }
        if registering.get() && name_value.is_empty() {
            error.set("Enter your name".to_string());
            return;
        }
        busy.set(true);
        error.set(String::new());
        let server_value = server.get().trim().trim_end_matches('/').to_string();
        let registering_value = registering.get();
        spawn(async move {
            let result = if registering_value {
                auth::register(&server_value, &name_value, &email_value, &password_value).await
            } else {
                auth::login(&server_value, &email_value, &password_value).await
            };
            match result {
                Ok(next) => {
                    server.set(server_value.clone());
                    edit_server.set(false);
                    busy.set(false);
                    if next.requires_verification {
                        verification.set(Some((email_value.clone(), next.token.clone())));
                        auth::clear_session();
                        return;
                    }
                    session.set(Session::SignedIn(next));
                    return;
                }
                Err(message) => error.set(message),
            }
            busy.set(false);
        });
    };

    view()
        .flex_1()
        .w_full()
        .items_center()
        .justify_center()
        .bg(move || theme().background)
        .p(px(16.0))
        .child(
            view()
                .w_full()
                .max_w(px(CARD_WIDTH))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .shadow([BoxShadow {
                    offset_y: px(1.0),
                    blur: px(2.0),
                    color: rgba(0x0000001a).into(),
                    ..BoxShadow::default()
                }])
                .child(
                    view()
                        .p(px(24.0))
                        .pb(px(0.0))
                        .gap(px(4.0))
                        .child(
                            text(move || {
                                if registering.get() {
                                    "Create account"
                                } else {
                                    "Sign in"
                                }
                            })
                            .text_size(px(24.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || {
                                if registering.get() {
                                    "Create an account on this server"
                                } else {
                                    "Enter your email and password to access your account"
                                }
                            })
                            .text_size(px(14.0))
                            .text_color(move || theme().muted_foreground),
                        ),
                )
                .when(!cfg!(target_arch = "wasm32"), |card| {
                    card.child(server_row(server, edit_server, error))
                })
                .child(
                    view()
                        .p(px(24.0))
                        .gap(px(16.0))
                        .child(
                            when(move || !error.get().is_empty(), move || error_box(error))
                                .otherwise(|| view().h(px(0.0))),
                        )
                        .child(
                            when(
                                move || registering.get(),
                                move || {
                                    form_field("Name", "Alex", "name", false, name, move || {
                                        submit()
                                    })
                                    .into_element()
                                },
                            )
                            .otherwise(|| view().h(px(0.0)).into_element()),
                        )
                        .child(form_field(
                            "Email",
                            "email@example.com",
                            "email",
                            false,
                            email,
                            move || submit(),
                        ))
                        .child(form_field(
                            "Password",
                            "",
                            "current-password",
                            true,
                            password,
                            move || submit(),
                        ))
                        .child(shared_button(
                            ButtonProps::new(if busy.get() {
                                "Please wait..."
                            } else if registering.get() {
                                "Create account"
                            } else {
                                "Sign in"
                            })
                            .full_width()
                            .disabled(busy.get())
                            .on_press(move || submit()),
                        ))
                        .child(shared_button(
                            ButtonProps::new("Use this device only")
                                .variant(crate::components::button::ButtonVariant::Outline)
                                .full_width()
                                .on_press(move || {
                                    crate::local::enable();
                                    let local_session = crate::local::session();
                                    crate::auth::store_session(&local_session);
                                    session.set(Session::SignedIn(local_session));
                                }),
                        ))
                        .child(
                            when(
                                move || demo_enabled.get(),
                                move || {
                                    shared_button(
                                        ButtonProps::new(if demo_loading.get() {
                                            "Loading..."
                                        } else {
                                            "Demo account"
                                        })
                                        .variant(crate::components::button::ButtonVariant::Outline)
                                        .full_width()
                                        .disabled(demo_loading.get())
                                        .on_press(move || demo()),
                                    )
                                    .into_element()
                                },
                            )
                            .otherwise(|| view().h(px(0.0))),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .items_center()
                                .justify_center()
                                .w_full()
                                .h(px(32.0))
                                .rounded(px(6.0))
                                .on_press(move || {
                                    registering.set(!registering.get());
                                    error.set(String::new());
                                })
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_center()
                                        .gap(px(4.0))
                                        .child(
                                            text(move || {
                                                if registering.get() {
                                                    "Already have an account?"
                                                } else {
                                                    "Don't have an account?"
                                                }
                                            })
                                            .text_size(px(14.0))
                                            .text_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            text(move || {
                                                if registering.get() {
                                                    "Sign in"
                                                } else {
                                                    "Sign up"
                                                }
                                            })
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || theme().primary),
                                        ),
                                ),
                        ),
                ),
        )
        .child(
            when(
                move || verification.get().is_some(),
                move || {
                    let (email, token) = verification.get().unwrap_or_default();
                    verification_dialog(session, server, email, token, password, verification)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

fn verification_dialog(
    session: Signal<Session>,
    server: Signal<String>,
    email: String,
    token: String,
    password: Signal<String>,
    verification: Signal<Option<(String, String)>>,
) -> impl IntoElement {
    let code = signal(String::new());
    let busy = signal(false);
    let error = signal(String::new());
    let status = signal(String::new());
    let verify = {
        let server = server.get();
        let token = token.clone();
        let email = email.clone();
        move || {
            if busy.get() {
                return;
            }
            let value = code.get().trim().to_string();
            if value.is_empty() {
                error.set("Enter the verification code.".to_string());
                return;
            }
            let server = server.clone();
            let token = token.clone();
            let email = email.clone();
            let password = password.get();
            busy.set(true);
            error.set(String::new());
            spawn(async move {
                match auth::verify_email(&server, &token, &value).await {
                    Ok(_) => {
                        if password.is_empty() {
                            status.set("Email verified. Sign in.".to_string());
                            verification.set(None);
                        } else {
                            match auth::login(&server, &email, &password).await {
                                Ok(next) => {
                                    verification.set(None);
                                    session.set(Session::SignedIn(next));
                                }
                                Err(message) => error.set(message),
                            }
                        }
                    }
                    Err(message) => error.set(message),
                }
                busy.set(false);
            });
        }
    };
    let resend = {
        let server = server.get();
        let token = token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match auth::resend_verification(&server, &token).await {
                    Ok(()) => status.set("A new code was sent.".to_string()),
                    Err(message) => error.set(message),
                }
            });
        }
    };
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Verify your email",
            format!("Enter the code sent to {email}."),
            move || verification.set(None),
        )
        .width(440.0)
        .body(
            view()
                .p(px(24.0))
                .gap(px(12.0))
                .child(crate::ui::field("Verification code", code, "123456"))
                .child(
                    text(move || error.get())
                        .text_size(px(11.0))
                        .text_color(move || theme().destructive),
                )
                .child(
                    text(move || status.get())
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                )
                .into_element(),
        )
        .footer(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .justify_end()
                .gap(px(8.0))
                .child(crate::ui::action_button("Back", false, move || {
                    verification.set(None)
                }))
                .child(crate::ui::action_button("Resend code", false, resend))
                .child(crate::ui::action_button(
                    if busy.get() { "Verifying…" } else { "Verify" },
                    false,
                    verify,
                ))
                .into_element(),
        ),
    )
}

fn server_row(
    server: Signal<String>,
    edit_server: Signal<bool>,
    error: Signal<String>,
) -> impl IntoElement {
    view()
        .mx(px(24.0))
        .mt(px(16.0))
        .gap(px(8.0))
        .child(
            view()
                .px(px(12.0))
                .py(px(8.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().muted)
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    view()
                        .flex_1()
                        .min_w(px(0.0))
                        .gap(px(2.0))
                        .child(
                            text("SERVER")
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text(move || server.get())
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground)
                                .number_of_lines(1),
                        ),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(32.0))
                        .px(px(12.0))
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().background)
                        .on_press(move || edit_server.set(!edit_server.get()))
                        .child(
                            text(if edit_server.get() { "Done" } else { "Change" })
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
        .child(
            when(
                move || edit_server.get(),
                move || {
                    input()
                        .value(server)
                        .on_change(move |value: &str| {
                            server.set(value.to_string());
                            error.set(String::new());
                        })
                        .placeholder("https://recurrency.example.com")
                        .keyboard_type(KeyboardType::Url)
                        .auto_fill(AutoFill::Off)
                        .text_size(px(13.0))
                        .h(px(34.0))
                        .px(px(10.0))
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || theme().input)
                        .bg(move || theme().background)
                        .text_color(move || theme().foreground)
                        .placeholder_color(move || theme().muted_foreground.dark)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

fn error_box(error: Signal<String>) -> impl IntoElement {
    view()
        .w_full()
        .rounded(px(6.0))
        .p(px(12.0))
        .bg(move || theme().destructive.opacity(0.12))
        .child(
            text(move || error.get())
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
}

fn form_field(
    label: &'static str,
    placeholder: &'static str,
    autofill: &'static str,
    masked: bool,
    value: Signal<String>,
    submit: impl Fn() + 'static,
) -> impl IntoElement {
    view()
        .w_full()
        .gap(px(6.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .child(
                    text(label)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("*")
                        .text_size(px(14.0))
                        .text_color(move || theme().destructive),
                ),
        )
        .child(
            input()
                .value(value)
                .on_change(move |text: &str| value.set(text.to_string()))
                .on_submit(move |_text: &str| submit())
                .placeholder(placeholder)
                .masked(masked)
                .auto_fill(match autofill {
                    "email" => AutoFill::Email,
                    "current-password" => AutoFill::Password,
                    _ => AutoFill::Off,
                })
                .keyboard_type(if autofill == "email" {
                    KeyboardType::Email
                } else {
                    KeyboardType::Default
                })
                .return_key_type(ReturnKeyType::Done)
                .text_size(px(14.0))
                .h(px(40.0))
                .w_full()
                .px(px(12.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().background)
                .text_color(move || theme().foreground)
                .placeholder_color(move || theme().muted_foreground.dark),
        )
}
