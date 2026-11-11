use crate::layout::app_layout::{Icon, svg_icon};
use crate::pages::settings_page::APP_VERSION;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Profile,
    Preferences,
    Money,
    Notifications,
}

impl SettingsSection {
    const ALL: [Self; 4] = [
        Self::Profile,
        Self::Preferences,
        Self::Money,
        Self::Notifications,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Profile => "Profile",
            Self::Preferences => "Preferences",
            Self::Money => "Money",
            Self::Notifications => "Notifications",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Profile => Icon::Settings,
            Self::Preferences => Icon::Sun,
            Self::Money => Icon::Categories,
            Self::Notifications => Icon::Bell,
        }
    }

    fn route(self) -> crate::Route {
        match self {
            Self::Profile => crate::Route::SettingsProfile {},
            Self::Preferences => crate::Route::SettingsPreferences {},
            Self::Money => crate::Route::SettingsMoney {},
            Self::Notifications => crate::Route::SettingsNotifications {},
        }
    }

    fn is_active(self, route: &crate::Route) -> bool {
        match self {
            Self::Profile => matches!(
                route,
                crate::Route::Settings {} | crate::Route::SettingsProfile {}
            ),
            Self::Preferences => matches!(route, crate::Route::SettingsPreferences {}),
            Self::Money => matches!(
                route,
                crate::Route::SettingsMoney {}
                    | crate::Route::SettingsAccounts {}
                    | crate::Route::SettingsCategories {}
                    | crate::Route::Accounts {}
                    | crate::Route::Categories {}
            ),
            Self::Notifications => matches!(route, crate::Route::SettingsNotifications {}),
        }
    }
}

#[component]
pub fn SettingsLayout() -> Element {
    let current: crate::Route = use_route();
    let nav = navigator();

    rsx! {
        div { class: "w-full px-4 lg:px-6 py-6 lg:py-8",
            div { class: "mx-auto flex w-full max-w-7xl gap-8 lg:gap-10",
                aside { class: "hidden lg:flex w-52 shrink-0 flex-col self-start sticky top-20",
                    p { class: "px-3 text-xs font-semibold uppercase tracking-widest text-muted-foreground/70", "Settings" }
                    nav { class: "mt-3 flex flex-col gap-1", aria_label: "Settings sections",
                        for section in SettingsSection::ALL {
                            SettingsSidebarLink {
                                section,
                                active: section.is_active(&current),
                            }
                        }
                    }
                    div { class: "mt-8 border-t border-border/50 pt-4",
                        button {
                            r#type: "button",
                            class: "flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors",
                            onclick: move |_| {
                                AUTH.write().logout();
                                AUTH.write().persist_to_storage();
                                nav.push(crate::Route::Login {});
                            },
                            {svg_icon(Icon::Logout, 18)}
                            "Log out"
                        }
                        p { class: "px-3 pt-4 text-xs text-muted-foreground/60",
                            {format!("Recurrency v{APP_VERSION}")}
                        }
                    }
                }

                div { class: "min-w-0 flex-1",
                    nav {
                        class: "lg:hidden sticky top-14 z-20 -mx-4 mb-6 border-b border-border/50 bg-background/95 px-4 backdrop-blur-xl",
                        aria_label: "Settings sections",
                        div { class: "flex gap-1 overflow-x-auto py-2",
                            for section in SettingsSection::ALL {
                                SettingsMobileLink {
                                    section,
                                    active: section.is_active(&current),
                                }
                            }
                        }
                    }
                    p { class: "lg:hidden px-1 pb-2 text-[11px] text-muted-foreground/60",
                        {format!("Recurrency v{APP_VERSION}")}
                    }
                    Outlet::<crate::Route> {}
                }
            }
        }
    }
}

#[component]
fn SettingsSidebarLink(section: SettingsSection, active: bool) -> Element {
    let nav = navigator();
    let to = section.route();
    let class = if active {
        "flex items-center gap-3 rounded-md bg-accent px-3 py-2 text-sm font-medium text-accent-foreground"
    } else {
        "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
    };
    rsx! {
        a {
            href: format!("{}", to),
            class,
            aria_current: if active { "page" } else { "false" },
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            {svg_icon(section.icon(), 18)}
            span { {section.label()} }
        }
    }
}

#[component]
fn SettingsMobileLink(section: SettingsSection, active: bool) -> Element {
    let nav = navigator();
    let to = section.route();
    let class = if active {
        "inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent px-3 py-1.5 text-xs font-semibold text-accent-foreground"
    } else {
        "inline-flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
    };
    rsx! {
        a {
            href: format!("{}", to),
            class,
            aria_current: if active { "page" } else { "false" },
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            {svg_icon(section.icon(), 15)}
            span { {section.label()} }
        }
    }
}
