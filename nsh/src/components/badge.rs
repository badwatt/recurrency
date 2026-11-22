//! Small status label shared by cards, rows and settings.

use nsh_base::theme;
use nsh_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeTone {
    #[default]
    Neutral,
    Primary,
    Success,
    Destructive,
    Outline,
}

pub struct BadgeProps {
    pub label: String,
    pub tone: BadgeTone,
}

impl BadgeProps {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tone: BadgeTone::Neutral,
        }
    }

    pub fn tone(mut self, tone: BadgeTone) -> Self {
        self.tone = tone;
        self
    }
}

pub fn badge(props: BadgeProps) -> impl IntoElement {
    let tone = props.tone;
    view()
        .items_center()
        .justify_center()
        .h(px(20.0))
        .px(px(7.0))
        .rounded(px(5.0))
        .border(px(1.0))
        .border_color(move || match tone {
            BadgeTone::Outline => theme().border,
            BadgeTone::Primary => theme().primary.opacity(0.35),
            BadgeTone::Success => theme().primary.opacity(0.35),
            BadgeTone::Destructive => theme().destructive.opacity(0.35),
            BadgeTone::Neutral => theme().border.opacity(0.0),
        })
        .bg(move || match tone {
            BadgeTone::Neutral => theme().muted,
            BadgeTone::Primary => theme().primary.opacity(0.12),
            BadgeTone::Success => theme().primary.opacity(0.12),
            BadgeTone::Destructive => theme().destructive.opacity(0.12),
            BadgeTone::Outline => theme().card,
        })
        .child(
            text(props.label)
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || match tone {
                    BadgeTone::Primary => theme().primary,
                    BadgeTone::Success => theme().primary,
                    BadgeTone::Destructive => theme().destructive,
                    _ => theme().muted_foreground,
                }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_defaults_to_a_neutral_tone() {
        let props = BadgeProps::new("Active");
        assert_eq!(props.label, "Active");
        assert_eq!(props.tone, BadgeTone::Neutral);
    }

    #[test]
    fn badge_tone_can_be_changed() {
        let props = BadgeProps::new("Active").tone(BadgeTone::Primary);
        assert_eq!(props.tone, BadgeTone::Primary);
    }
}
