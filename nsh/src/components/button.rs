//! Shared button primitives. Every press target in the app goes through this
//! module so height, radius, typography and disabled states stay identical.

use nsh_base::{button as base_button, theme};
use nsh_core::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Unstyled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Icon,
    IconSm,
    Unstyled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonAlign {
    #[default]
    Center,
    Start,
    End,
    Between,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonRadius {
    None,
    Sm,
    #[default]
    Md,
    Lg,
    Full,
}

impl ButtonSize {
    fn height(self) -> f32 {
        match self {
            Self::Xs => 26.0,
            Self::Sm => 34.0,
            Self::Md => 40.0,
            Self::Lg => 48.0,
            Self::Icon | Self::IconSm | Self::Unstyled => 0.0,
        }
    }

    fn width(self) -> f32 {
        match self {
            Self::Icon => 40.0,
            Self::IconSm => 30.0,
            _ => 0.0,
        }
    }

    fn padding(self) -> f32 {
        match self {
            Self::Xs => 8.0,
            Self::Sm => 10.0,
            Self::Md => 14.0,
            Self::Lg => 18.0,
            Self::Icon | Self::IconSm | Self::Unstyled => 0.0,
        }
    }

    fn text(self) -> f32 {
        match self {
            Self::Xs | Self::Sm => 12.0,
            Self::Lg => 15.0,
            _ => 13.0,
        }
    }
}

impl ButtonRadius {
    fn value(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Sm => 5.0,
            Self::Md => 7.0,
            Self::Lg => 10.0,
            Self::Full => 999.0,
        }
    }
}

/// Props for [`button`]. `on_press` is an `Rc` so the same builder can be
/// reused by rows and cards without cloning owned strings into each closure.
pub struct ButtonProps {
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub align: ButtonAlign,
    pub radius: ButtonRadius,
    pub icon: Option<ImageAsset>,
    pub disabled: bool,
    pub selected: bool,
    pub full_width: bool,
    pub accessibility_label: Option<String>,
    pub on_press: Option<Rc<dyn Fn()>>,
}

impl ButtonProps {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            align: ButtonAlign::Center,
            radius: ButtonRadius::Md,
            icon: None,
            disabled: false,
            selected: false,
            full_width: false,
            accessibility_label: None,
            on_press: None,
        }
    }

    pub fn icon(mut self, icon: ImageAsset) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn align(mut self, align: ButtonAlign) -> Self {
        self.align = align;
        self
    }

    pub fn radius(mut self, radius: ButtonRadius) -> Self {
        self.radius = radius;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn on_press(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_press = Some(Rc::new(handler));
        self
    }
}

pub fn button(props: ButtonProps) -> impl IntoElement {
    let ButtonProps {
        label,
        variant,
        size,
        align: _align,
        radius,
        icon,
        disabled,
        selected,
        full_width,
        accessibility_label,
        on_press,
    } = props;
    let a11y_label = accessibility_label.unwrap_or_else(|| label.clone());
    let width = size.width();
    let height = size.height();
    let padding = size.padding();
    let label_size = size.text();
    let label_color = move || match variant {
        ButtonVariant::Primary => theme().primary_foreground,
        ButtonVariant::Destructive => theme().foreground,
        _ => theme().foreground,
    };
    base_button()
        .flex_row()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .h(px(height))
        .when(width > 0.0, |button| button.w(px(width)))
        .when(full_width, |button| button.w_full())
        .px(px(padding))
        .rounded(px(radius.value()))
        .border(px(1.0))
        .border_color(move || if selected {
            theme().primary
        } else {
            match variant {
                ButtonVariant::Outline => theme().border,
                _ => theme().border.opacity(0.0),
            }
        })
        .bg(move || if selected {
            theme().accent
        } else {
            match variant {
                ButtonVariant::Primary => theme().primary,
                ButtonVariant::Secondary => theme().secondary,
                ButtonVariant::Outline | ButtonVariant::Ghost => theme().card,
                ButtonVariant::Destructive => theme().destructive,
                ButtonVariant::Unstyled => theme().card.opacity(0.0),
            }
        })
        .opacity(if disabled { 0.5 } else { 1.0 })
        .disabled(disabled)
        .accessibility_label(a11y_label)
        .on_press(move || {
            if let Some(handler) = &on_press {
                handler();
            }
        })
        .child(icon.map_or_else(|| view().into_element(), |icon| {
            image(icon).size(px(label_size + 3.0)).tint_color(move || match variant {
                ButtonVariant::Primary => theme().primary_foreground,
                ButtonVariant::Destructive => theme().foreground,
                _ => theme().foreground,
            }).into_element()
        }))
        .when(!label.is_empty(), |button| {
            button.child(
                text(label)
                    .number_of_lines(1)
                    .text_size(px(label_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(label_color),
            )
        })
}

pub fn icon_button(icon: ImageAsset, label: &'static str, active: bool, on_press: impl Fn() + 'static) -> impl IntoElement {
    button(
        ButtonProps::new("")
            .icon(icon)
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Ghost)
            .selected(active)
            .accessibility_label(label)
            .on_press(on_press),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_defaults_to_primary_medium() {
        let props = ButtonProps::new("Save");
        assert_eq!(props.variant, ButtonVariant::Primary);
        assert_eq!(props.size, ButtonSize::Md);
        assert_eq!(props.radius, ButtonRadius::Md);
        assert_eq!(props.label, "Save");
    }

    #[test]
    fn button_sizes_keep_the_shared_scale() {
        assert_eq!(ButtonSize::Xs.height(), 26.0);
        assert_eq!(ButtonSize::Md.height(), 40.0);
        assert_eq!(ButtonSize::Icon.width(), 40.0);
    }
}
