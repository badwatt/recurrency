//! Circular identity badge shared by the top bar, account settings and member
//! lists. Uses the uploaded avatar when present, initials otherwise.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn avatar_badge(size: f32, name: &str, avatar: Option<String>) -> impl IntoElement {
    let words: Vec<&str> = name.split_whitespace().collect();
    let initial = match words.as_slice() {
        [] => "?".to_string(),
        [single] => single
            .chars()
            .next()
            .map(|character| character.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string()),
        [first, .., last] => {
            let first = first.chars().next().unwrap_or('?');
            let last = last.chars().next().unwrap_or('?');
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        }
    };
    let avatar = avatar.filter(|value| !value.trim().is_empty());
    let has_avatar = avatar.is_some();
    view()
        .h(px(size))
        .w(px(size))
        .items_center()
        .justify_center()
        .rounded(px(999.0))
        .overflow_hidden()
        .bg(move || {
            if has_avatar {
                theme().muted
            } else {
                theme().primary
            }
        })
        .when(has_avatar, |view| {
            view.child(image(avatar.clone().unwrap_or_default()).size(px(size)))
        })
        .when(!has_avatar, |view| {
            view.child(
                text(initial)
                    .text_size(px(size * 0.36))
                    .font_weight(FontWeight::BOLD)
                    .text_color(move || theme().primary_foreground),
            )
        })
}
