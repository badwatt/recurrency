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
    Toolbar,
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
            Self::Sm => 38.0,
            Self::Md => 40.0,
            Self::Lg => 48.0,
            Self::Icon => 40.0,
            Self::IconSm => 30.0,
            Self::Toolbar => 36.0,
            Self::Unstyled => 0.0,
        }
    }

    fn width(self) -> f32 {
        match self {
            Self::Icon => 40.0,
            Self::IconSm => 30.0,
            Self::Toolbar => 36.0,
            _ => 0.0,
        }
    }

    fn padding(self) -> f32 {
        match self {
            Self::Xs => 8.0,
            Self::Sm => 12.0,
            Self::Md => 16.0,
            Self::Lg => 18.0,
            Self::Icon | Self::IconSm | Self::Toolbar | Self::Unstyled => 0.0,
        }
    }

    fn text(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm => 14.0,
            Self::Lg => 15.0,
            _ => 14.0,
        }
    }
}

impl ButtonRadius {
    fn value(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Sm => 5.0,
            Self::Md => 6.0,
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
    pub trailing_icon: Option<ImageAsset>,
    pub disabled: bool,
    pub selected: bool,
    pub selected_when: Option<Rc<dyn Fn() -> bool>>,
    pub full_width: bool,
    pub min_width: Option<f32>,
    pub fixed_width: Option<f32>,
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
            trailing_icon: None,
            disabled: false,
            selected: false,
            selected_when: None,
            full_width: false,
            min_width: None,
            fixed_width: None,
            accessibility_label: None,
            on_press: None,
        }
    }

    pub fn icon(mut self, icon: ImageAsset) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_opt(mut self, icon: Option<ImageAsset>) -> Self {
        self.icon = icon;
        self
    }

    pub fn trailing_icon(mut self, icon: ImageAsset) -> Self {
        self.trailing_icon = Some(icon);
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

    pub fn selected_when(mut self, selected: impl Fn() -> bool + 'static) -> Self {
        self.selected_when = Some(Rc::new(selected));
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

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn fixed_width(mut self, width: f32) -> Self {
        self.fixed_width = Some(width);
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
        trailing_icon,
        disabled,
        selected,
        selected_when,
        full_width,
        min_width,
        fixed_width,
        accessibility_label,
        on_press,
    } = props;
    let a11y_label = accessibility_label.unwrap_or_else(|| label.clone());
    let width = size.width();
    let height = size.height();
    let padding = size.padding();
    let label_size = size.text();
    let icon_size = match size {
        ButtonSize::Icon => 18.0,
        ButtonSize::IconSm | ButtonSize::Toolbar => 16.0,
        _ => label_size + 3.0,
    };
    let selected_border = selected_when.clone();
    let selected_fill = selected_when.clone();
    let selected_icon = selected_when;
    let label_color = move || match variant {
        ButtonVariant::Primary => theme().primary_foreground,
        ButtonVariant::Destructive => theme().foreground,
        _ => theme().foreground,
    };
    base_button()
        .cursor_pointer()
        .flex_row()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .self_start()
        .h(px(height))
        .when(width > 0.0, |button| button.w(px(width)))
        .when(full_width, |button| button.w_full())
        .when_some(min_width, |button, width| button.min_w(px(width)))
        .when_some(fixed_width, |button, width| button.w(px(width)))
        .px(px(padding))
        .rounded(px(radius.value()))
        .border(px(1.0))
        .border_color(move || {
            if selected_border.as_ref().map_or(selected, |check| check()) {
                theme().primary
            } else {
                match variant {
                    ButtonVariant::Outline => theme().border,
                    _ => theme().border.opacity(0.0),
                }
            }
        })
        .bg(move || {
            if selected_fill.as_ref().map_or(selected, |check| check()) {
                theme().accent
            } else {
                match variant {
                    ButtonVariant::Primary => theme().primary,
                    ButtonVariant::Secondary => theme().secondary,
                    ButtonVariant::Outline | ButtonVariant::Ghost => theme().card,
                    ButtonVariant::Destructive => theme().destructive,
                    ButtonVariant::Unstyled => theme().card.opacity(0.0),
                }
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
        .child(icon.map_or_else(
            || view().into_element(),
            |icon| {
                image(icon)
                    .size(px(icon_size))
                    .pointer_events(PointerEvents::None)
                    .tint_color(move || match variant {
                        ButtonVariant::Primary => theme().primary_foreground,
                        ButtonVariant::Outline
                            if !selected_icon.as_ref().map_or(selected, |check| check()) =>
                        {
                            theme().muted_foreground
                        }
                        ButtonVariant::Destructive => theme().foreground,
                        _ => theme().foreground,
                    })
                    .into_element()
            },
        ))
        .when(!label.is_empty(), |button| {
            button.child(
                text(label)
                    .number_of_lines(1)
                    .text_size(px(label_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(label_color),
            )
        })
        .child(trailing_icon.map_or_else(
            || view().into_element(),
            |icon| {
                image(icon)
                    .size(px(icon_size))
                    .ml(px(6.0))
                    .pointer_events(PointerEvents::None)
                    .tint_color(move || match variant {
                        ButtonVariant::Primary => theme().primary_foreground,
                        _ => theme().muted_foreground,
                    })
                    .into_element()
            },
        ))
}

pub fn icon_button(
    icon: ImageAsset,
    label: &'static str,
    active: bool,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button(
        ButtonProps::new("")
            .icon(icon)
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Outline)
            .selected(active)
            .accessibility_label(label)
            .on_press(on_press),
    )
}

pub fn icon_button_when(
    icon: ImageAsset,
    label: &'static str,
    active: impl Fn() -> bool + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button(
        ButtonProps::new("")
            .icon(icon)
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Outline)
            .selected_when(active)
            .accessibility_label(label)
            .on_press(on_press),
    )
}

pub fn icon_button_when_sized(
    icon: ImageAsset,
    label: &'static str,
    size: ButtonSize,
    active: impl Fn() -> bool + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button(
        ButtonProps::new("")
            .icon(icon)
            .size(size)
            .variant(ButtonVariant::Outline)
            .selected_when(active)
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
        assert_eq!(ButtonSize::Icon.height(), 40.0);
    }
}
