use crate::Route;
use crate::state::app_mode::APP_MODE;
use dioxus::prelude::*;

/// Connection mode card. Stateless: reads the global app mode and can send the
/// user back to Setup to switch between offline and server modes.
#[component]
pub fn ConnectionCard() -> Element {
    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Server" }
                p { class: "text-sm text-muted-foreground",
                    if APP_MODE().mode.is_offline() {
                        "Offline mode — data stored on this device"
                    } else {
                        "Connected to your Recurrency server"
                    }
                }
            }
            div { class: "p-6 space-y-4",
                // Current mode indicator
                div { class: "flex items-center gap-3",
                    div { class: "flex h-9 w-9 items-center justify-center rounded-lg",
                        if APP_MODE().mode.is_offline() {
                            // Database icon
                            svg {
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "h-5 w-5 text-primary",
                                path { d: "M12 2L2 7l10 5 10-5-10-5z" }
                                path { d: "M2 17l10 5 10-5" }
                                path { d: "M2 12l10 5 10-5" }
                            }
                        } else {
                            // Cloud icon
                            svg {
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "h-5 w-5 text-blue-500",
                                circle { cx: "12", cy: "12", r: "3" }
                                path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
                            }
                        }
                    }
                    div { class: "flex-1",
                        p { class: "text-sm font-medium",
                            if APP_MODE().mode.is_offline() {
                                "Offline Mode"
                            } else {
                                "Server Mode"
                            }
                        }
                        if !APP_MODE().mode.is_offline() {
                            p { class: "text-xs text-muted-foreground",
                                "{APP_MODE().mode.server_url().unwrap_or_default()}"
                            }
                        }
                    }
                }
                // Change mode button
                {
                    let nav = navigator();
                    rsx! {
                        button {
                            onclick: move |_| {
                                nav.push(Route::Setup {});
                            },
                            class: "w-full rounded-md border border-border bg-background px-4 py-2 text-sm font-medium text-foreground hover:bg-accent transition-colors",
                            "Change connection mode"
                        }
                    }
                }
            }
        }
    }
}
