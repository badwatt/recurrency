//! Native sign-in surface, matching the Dioxus card layout.

use crate::auth::{self, Session};
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
                    server.set(server_value);
                    edit_server.set(false);
                    busy.set(false);
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
                            text(move || if registering.get() { "Create account" } else { "Sign in" })
                                .text_size(px(24.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(move || if registering.get() {
                                "Create an account on this server"
                            } else {
                                "Enter your email and password to access your account"
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
                        .when(registering.get(), move |form| {
                            form.child(form_field("Name", "Alex", "name", false, name, move || submit()))
                        })
                        .child(form_field("Email", "email@example.com", "email", false, email, move || submit()))
                        .child(form_field("Password", "", "current-password", true, password, move || submit()))
                        .child(
                            button()
                                .items_center()
                                .justify_center()
                                .w_full()
                                .h(px(40.0))
                                .rounded(px(6.0))
                                .bg(move || theme().primary)
                                .active(|s| s.opacity(0.85))
                                .disabled(move || busy.get())
                                .disabled_style(|s| s.opacity(0.5))
                                .on_press(move || submit())
                                .child(
                                    text(move || {
                                        if busy.get() {
                                            "Please wait..."
                                        } else if registering.get() {
                                            "Create account"
                                        } else {
                                            "Sign in"
                                        }
                                    })
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().primary_foreground),
                                ),
                        )
                        .child(
                            when(move || demo_enabled.get(), move || {
                                button()
                                    .items_center()
                                    .justify_center()
                                    .w_full()
                                    .h(px(40.0))
                                    .rounded(px(6.0))
                                    .border(px(1.0))
                                    .border_color(move || theme().border)
                                    .bg(move || theme().background)
                                    .disabled(move || demo_loading.get())
                                    .disabled_style(|state| state.opacity(0.5))
                                    .on_press(move || demo())
                                    .child(
                                        text(move || {
                                            if demo_loading.get() { "Loading..." } else { "Demo account" }
                                        })
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || theme().foreground),
                                    )
                                    .into_element()
                            })
                            .otherwise(|| view().h(px(0.0))),
                        )
                        .child(
                            button()
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
                                            text(move || if registering.get() {
                                                "Already have an account?"
                                            } else {
                                                "Don't have an account?"
                                            })
                                            .text_size(px(14.0))
                                            .text_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            text(move || if registering.get() { "Sign in" } else { "Sign up" })
                                                .text_size(px(14.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || theme().primary),
                                        ),
                                ),
                        ),
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
        .when(edit_server.get(), |row| {
            row.child(
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
                    .placeholder_color(move || theme().muted_foreground.dark),
            )
        })
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
