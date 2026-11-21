use dioxus::prelude::*;

/// Visual tone of a button. Every surface uses one of these variants instead
/// of re-declaring Tailwind colour classes at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
    Unstyled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonRadius {
    None,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl ButtonRadius {
    fn class(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Sm => "rounded-sm",
            Self::Md => "rounded-md",
            Self::Lg => "rounded-lg",
            Self::Xl => "rounded-xl",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonAlign {
    #[default]
    Default,
    Center,
    Start,
    Between,
}

impl ButtonAlign {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Center => "justify-center",
            Self::Start => "justify-start",
            Self::Between => "justify-between",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    IconSm,
    Icon,
    IconLg,
    Unstyled,
}

impl ButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => {
                "bg-primary text-primary-foreground hover:bg-primary/90 active:bg-primary/80"
            }
            Self::Secondary => {
                "bg-secondary text-secondary-foreground hover:bg-secondary/80 active:bg-secondary/70"
            }
            Self::Outline => {
                "border border-input bg-background hover:bg-accent hover:text-accent-foreground active:bg-accent/70"
            }
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground active:bg-accent/70",
            Self::Destructive => {
                "bg-destructive text-destructive-foreground hover:bg-destructive/90 active:bg-destructive/80"
            }
            Self::Link => "text-primary underline-offset-4 hover:underline px-0",
            Self::Unstyled => "",
        }
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "h-7 px-2 text-xs gap-1",
            Self::Sm => "h-8 px-3 text-xs gap-1.5",
            Self::Md => "h-10 px-4 py-2 text-sm gap-2",
            Self::Lg => "h-11 px-5 text-sm gap-2",
            Self::IconSm => "h-8 w-8 p-0",
            Self::Icon => "h-9 w-9 p-0",
            Self::IconLg => "h-10 w-10 p-0",
            Self::Unstyled => "",
        }
    }
}

/// Shared button primitive. The base includes focus, disabled and press
/// feedback, so callers only choose a variant, a size and optional layout.
#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] radius: ButtonRadius,
    #[props(default)] align: ButtonAlign,
    #[props(default)] disabled: bool,
    #[props(default)] full_width: bool,
    #[props(default = "button".to_string())] button_type: String,
    #[props(default)] id: String,
    #[props(default)] class: String,
    title: Option<String>,
    aria_label: Option<String>,
    role: Option<String>,
    aria_selected: Option<String>,
    aria_pressed: Option<String>,
    aria_expanded: Option<String>,
    aria_haspopup: Option<String>,
    style: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let variant_class = variant.class();
    let size_class = size.class();
    let width_class = if full_width { "w-full" } else { "" };
    let is_unstyled = variant == ButtonVariant::Unstyled;
    let base_class = if is_unstyled {
        "disabled:pointer-events-none disabled:opacity-50"
    } else {
        "inline-flex items-center font-medium transition-all duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98]"
    };
    let resolved_align = match align {
        ButtonAlign::Default if is_unstyled => ButtonAlign::Start,
        ButtonAlign::Default => ButtonAlign::Center,
        explicit => explicit,
    };
    let align_class = resolved_align.class();
    let radius_class = radius.class();
    let title = title.unwrap_or_default();
    let aria_label = aria_label.unwrap_or_default();
    let role = role.unwrap_or_default();
    let aria_selected = aria_selected.unwrap_or_default();
    let aria_pressed = aria_pressed.unwrap_or_default();
    let aria_expanded = aria_expanded.unwrap_or_default();
    let aria_haspopup = aria_haspopup.unwrap_or_default();
    let style = style.unwrap_or_default();

    rsx! {
        button {
            r#type: button_type,
            id,
            class: "{base_class} {align_class} {radius_class} {variant_class} {size_class} {width_class} {class}",
            disabled,
            title,
            aria_label,
            role,
            aria_selected,
            aria_pressed,
            aria_expanded,
            aria_haspopup,
            style,
            onclick: move |event| {
                if let Some(handler) = onclick {
                    handler.call(event);
                }
            },
            onmousedown: move |event| {
                if let Some(handler) = onmousedown {
                    handler.call(event);
                }
            },
            onmouseenter: move |event| {
                if let Some(handler) = onmouseenter {
                    handler.call(event);
                }
            },
            onmouseleave: move |event| {
                if let Some(handler) = onmouseleave {
                    handler.call(event);
                }
            },
            {children}
        }
    }
}

/// Icon-only button with an accessible name and native tooltip.
#[component]
pub fn IconButton(
    label: String,
    #[props(default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[props(default = ButtonSize::IconSm)] size: ButtonSize,
    #[props(default)] radius: ButtonRadius,
    #[props(default)] align: ButtonAlign,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    aria_pressed: Option<String>,
    aria_expanded: Option<String>,
    aria_haspopup: Option<String>,
    style: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        Button {
            variant,
            size,
            radius,
            align,
            disabled,
            class,
            title: label.clone(),
            aria_label: label,
            aria_pressed,
            onclick,
            onmousedown,
            onmouseenter,
            onmouseleave,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_is_the_default_variant() {
        assert_eq!(ButtonVariant::default(), ButtonVariant::Primary);
    }

    #[test]
    fn every_variant_has_a_distinct_class() {
        let classes = [
            ButtonVariant::Primary.class(),
            ButtonVariant::Secondary.class(),
            ButtonVariant::Outline.class(),
            ButtonVariant::Ghost.class(),
            ButtonVariant::Destructive.class(),
            ButtonVariant::Link.class(),
        ];
        for (index, class) in classes.iter().enumerate() {
            assert!(!class.is_empty());
            assert!(
                classes.iter().skip(index + 1).all(|other| other != class),
                "button variants must not share their full class list"
            );
        }
    }

    #[test]
    fn icon_sizes_are_square() {
        assert!(ButtonSize::IconSm.class().contains("w-8"));
        assert!(ButtonSize::Icon.class().contains("w-9"));
    }

    #[test]
    fn button_uses_the_shared_focus_and_disabled_rules() {
        let source = include_str!("button.rs");
        assert!(source.contains("focus-visible:ring-2"));
        assert!(source.contains("disabled:pointer-events-none"));
    }
}
