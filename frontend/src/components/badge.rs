use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
    Muted,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "bg-primary text-primary-foreground",
            Self::Secondary => "bg-secondary text-secondary-foreground",
            Self::Destructive => "bg-destructive text-destructive-foreground",
            Self::Outline => "border border-border text-foreground",
            Self::Success => "bg-emerald-600/15 text-emerald-700 dark:text-emerald-300",
            Self::Warning => "bg-amber-500/15 text-amber-700 dark:text-amber-300",
            Self::Muted => "bg-muted text-muted-foreground",
        }
    }
}

#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-semibold transition-colors {variant.class()} {class}",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_defaults_to_primary() {
        assert_eq!(BadgeVariant::default(), BadgeVariant::Primary);
    }

    #[test]
    fn badge_variants_cover_status_semantics() {
        assert!(BadgeVariant::Success.class().contains("emerald"));
        assert!(BadgeVariant::Warning.class().contains("amber"));
        assert!(BadgeVariant::Destructive.class().contains("destructive"));
    }
}
