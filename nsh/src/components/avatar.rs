//! Circular identity badge shared by the top bar, account settings and member
//! lists. Uses the uploaded avatar when present, initials otherwise.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn avatar_badge(
    size: f32,
    name: &str,
    email: &str,
    avatar: Option<String>,
) -> impl IntoElement {
    let words: Vec<&str> = name.split_whitespace().collect();
    let source = if name.trim().is_empty() {
        email.split('@').next().unwrap_or("?")
    } else {
        name
    };
    let words: Vec<&str> = source.split_whitespace().collect();
    let initial = match words.as_slice() {
        [] => "?".to_string(),
        [single] => single.chars().take(2).collect::<String>().to_uppercase(),
        [first, .., last] => {
            let first = first.chars().next().unwrap_or('?');
            let last = last.chars().next().unwrap_or('?');
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        }
    };
    const COLORS: [u32; 8] = [
        0x6366f1, 0x8b5cf6, 0xec4899, 0xf97316, 0x14b8a6, 0x0ea5e9, 0x22c55e, 0xeab308,
    ];
    let hash = email
        .bytes()
        .fold(0usize, |acc, byte| acc.wrapping_add(usize::from(byte)));
    let color = COLORS[hash % COLORS.len()];
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
                nsh_core::element::AdaptiveColor::rgb(color, color)
            }
        })
        .when(has_avatar, |view| {
            view.child(image(avatar.clone().unwrap_or_default()).size(px(size)))
        })
        .when(!has_avatar, |view| {
            view.child(
                text(initial)
                    .text_size(px(size * 0.34))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(move || rgb(0xFFFFFF)),
            )
        })
}
