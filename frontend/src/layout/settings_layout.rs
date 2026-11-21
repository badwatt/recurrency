use crate::components::button::{Button, ButtonAlign, ButtonVariant};
use crate::layout::app_layout::{Icon, svg_icon};
use crate::pages::settings_page::APP_VERSION;
use crate::state::auth::AUTH;
use crate::utils::scroll::scroll_to_top;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Account,
    Appearance,
    General,
    Ai,
    Money,
    Notifications,
    Data,
    Server,
    About,
    Admin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsGroup {
    Personal,
    Configuration,
    System,
}

impl SettingsGroup {
    const ALL: [Self; 3] = [Self::Personal, Self::Configuration, Self::System];

    fn label(self) -> &'static str {
        match self {
            Self::Personal => "Personal",
            Self::Configuration => "Configuration",
            Self::System => "System",
        }
    }

    fn sections(self) -> &'static [SettingsSection] {
        match self {
            Self::Personal => &[SettingsSection::Account, SettingsSection::Appearance],
            Self::Configuration => &[
                SettingsSection::General,
                SettingsSection::Ai,
                SettingsSection::Money,
                SettingsSection::Notifications,
            ],
            Self::System => &[
                SettingsSection::Data,
                SettingsSection::Server,
                SettingsSection::Admin,
                SettingsSection::About,
            ],
        }
    }
}

impl SettingsSection {
    const ALL: [Self; 10] = [
        Self::Account,
        Self::Appearance,
        Self::General,
        Self::Ai,
        Self::Money,
        Self::Notifications,
        Self::Data,
        Self::Server,
        Self::Admin,
        Self::About,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Appearance => "Appearance",
            Self::General => "General",
            Self::Ai => "AI assistant",
            Self::Money => "Accounts & categories",
            Self::Notifications => "Notifications",
            Self::Data => "Data & backup",
            Self::Server => "Server",
            Self::About => "About",
            Self::Admin => "Administration",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Account => "Profile, session and account removal",
            Self::Appearance => "Color themes and palette",
            Self::General => "Currency, exchange rates, week start and dates",
            Self::Ai => "Model, voice input and privacy",
            Self::Money => "Accounts and categories",
            Self::Notifications => "Delivery channels and summaries",
            Self::Data => "Backups, restore and folder sync",
            Self::Server => "Server connection and device pairing",
            Self::About => "Version, release notes and Android app",
            Self::Admin => "Users, registration and full backups",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Account => Icon::Settings,
            Self::Appearance => Icon::Paintbrush,
            Self::General => Icon::Sun,
            Self::Ai => Icon::Ai,
            Self::Money => Icon::Categories,
            Self::Notifications => Icon::Bell,
            Self::Data => Icon::Reports,
            Self::Server => Icon::Monitor,
            Self::About => Icon::Info,
            Self::Admin => Icon::Admin,
        }
    }

    fn route(self) -> crate::Route {
        match self {
            Self::Account => crate::Route::SettingsProfile {},
            Self::Appearance => crate::Route::SettingsAppearance {},
            Self::General => crate::Route::SettingsPreferences {},
            Self::Ai => crate::Route::SettingsAi {},
            Self::Money => crate::Route::SettingsMoney {},
            Self::Notifications => crate::Route::SettingsNotifications {},
            Self::Data => crate::Route::SettingsData {},
            Self::Server => crate::Route::SettingsServer {},
            Self::About => crate::Route::SettingsAboutPage {},
            Self::Admin => crate::Route::SettingsAdmin {},
        }
    }

    fn visible_to(self, is_admin: bool) -> bool {
        self != Self::Admin || is_admin
    }

    fn is_active(self, route: &crate::Route) -> bool {
        match self {
            Self::Account => matches!(
                route,
                crate::Route::Settings {} | crate::Route::SettingsProfile {}
            ),
            Self::Appearance => matches!(route, crate::Route::SettingsAppearance {}),
            Self::General => matches!(route, crate::Route::SettingsPreferences {}),
            Self::Ai => matches!(route, crate::Route::SettingsAi {}),
            Self::Money => matches!(
                route,
                crate::Route::SettingsMoney {}
                    | crate::Route::SettingsAccounts {}
                    | crate::Route::SettingsCategories {}
                    | crate::Route::Accounts {}
                    | crate::Route::Categories {}
            ),
            Self::Notifications => matches!(route, crate::Route::SettingsNotifications {}),
            Self::Data => matches!(route, crate::Route::SettingsData {}),
            Self::Server => matches!(route, crate::Route::SettingsServer {}),
            Self::About => matches!(route, crate::Route::SettingsAboutPage {}),
            Self::Admin => matches!(route, crate::Route::SettingsAdmin {}),
        }
    }
}

#[component]
pub fn SettingsLayout() -> Element {
    let current: crate::Route = use_route();
    let nav = navigator();
    let is_admin = AUTH().user.as_ref().is_some_and(|user| user.is_admin);

    rsx! {
        div { class: "w-full px-4 lg:px-6 py-6 lg:py-8",
            div { class: "mx-auto flex w-full max-w-7xl gap-8 lg:gap-10",
                aside { class: "hidden lg:flex w-52 shrink-0 flex-col self-start sticky top-20",
                    p { class: "px-3 text-xs font-semibold uppercase tracking-widest text-muted-foreground/70", "Settings" }
                    nav { class: "mt-3 flex flex-col gap-4", aria_label: "Settings sections",
                        for group in SettingsGroup::ALL {
                            div { class: "flex flex-col gap-1",
                                p { class: "px-3 pb-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground/50",
                                    {group.label()}
                                }
                                for section in group.sections() {
                                    if section.visible_to(is_admin) {
                                        SettingsSidebarLink {
                                            section: *section,
                                            active: section.is_active(&current),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "mt-8 border-t border-border/50 pt-4",
                        Button {
                            variant: ButtonVariant::Ghost,
                            full_width: true,
                            align: ButtonAlign::Start,
                            class: "px-3 text-muted-foreground hover:bg-destructive/10 hover:text-destructive".to_string(),
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
                                if section.visible_to(is_admin) {
                                    SettingsMobileLink {
                                        section,
                                        active: section.is_active(&current),
                                    }
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
        "flex items-start gap-3 rounded-lg border border-primary/20 bg-accent px-3 py-2.5 text-sm font-medium text-accent-foreground"
    } else {
        "flex items-start gap-3 rounded-lg border border-transparent px-3 py-2.5 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 hover:border-border/50 transition-colors"
    };
    rsx! {
        a {
            href: format!("{}", to),
            class,
            aria_current: if active { "page" } else { "false" },
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
                scroll_to_top();
            },
            span { class: "mt-0.5 shrink-0", {svg_icon(section.icon(), 17)} }
            span { class: "min-w-0 flex flex-col gap-0.5",
                span { {section.label()} }
                span { class: "text-[10px] font-normal leading-snug text-muted-foreground/80", {section.description()} }
            }
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
                scroll_to_top();
            },
            {svg_icon(section.icon(), 15)}
            span { {section.label()} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_cover_all_groups_and_labels() {
        assert_eq!(SettingsSection::ALL.len(), 10);
        let mut sections = Vec::new();
        for group in SettingsGroup::ALL {
            assert!(!group.label().is_empty());
            sections.extend_from_slice(group.sections());
        }
        for section in SettingsSection::ALL {
            assert!(!section.label().is_empty());
            assert!(!section.description().is_empty());
            assert!(sections.contains(&section));
            let _ = section.icon();
        }
    }

    #[test]
    fn admin_section_is_hidden_from_members() {
        assert!(SettingsSection::Admin.visible_to(true));
        assert!(!SettingsSection::Admin.visible_to(false));
        for section in SettingsSection::ALL {
            assert!(section.visible_to(true));
        }
    }

    #[test]
    fn routes_map_to_the_matching_section() {
        assert!(SettingsSection::Account.is_active(&crate::Route::SettingsProfile {}));
        assert!(SettingsSection::Appearance.is_active(&crate::Route::SettingsAppearance {}));
        assert!(SettingsSection::General.is_active(&crate::Route::SettingsPreferences {}));
        assert!(SettingsSection::Ai.is_active(&crate::Route::SettingsAi {}));
        assert!(SettingsSection::Money.is_active(&crate::Route::SettingsAccounts {}));
        assert!(SettingsSection::Money.is_active(&crate::Route::SettingsCategories {}));
        assert!(SettingsSection::Notifications.is_active(&crate::Route::SettingsNotifications {}));
        assert!(SettingsSection::Data.is_active(&crate::Route::SettingsData {}));
        assert!(SettingsSection::Server.is_active(&crate::Route::SettingsServer {}));
        assert!(SettingsSection::About.is_active(&crate::Route::SettingsAboutPage {}));
        assert!(SettingsSection::Admin.is_active(&crate::Route::SettingsAdmin {}));
        assert!(!SettingsSection::General.is_active(&crate::Route::SettingsData {}));
    }
}
