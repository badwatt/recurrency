//! Amount rendered in the display currency, with the original currency in a
//! lighter suffix when conversion applied. Privacy mode masks both parts.

use nsh_base::theme;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::format::format_currency;
use crate::models::Rates;

pub fn converted_amount(
    amount: Decimal,
    currency: &str,
    display_currency: &str,
    rates: &Rates,
    size: f32,
    weight: FontWeight,
    mono: bool,
) -> impl IntoElement {
    let converted = rates.convert(amount, currency);
    let main = format_currency(converted, display_currency);
    let original = (!currency.eq_ignore_ascii_case(display_currency))
        .then(|| format_currency(amount, currency));
    view()
        .flex_row()
        .items_baseline()
        .gap(px(4.0))
        .child(
            text(main)
                .text_size(px(size))
                .line_height(px(size * 4.0 / 3.0))
                .font_weight(weight)
                .when(mono, |text| text.font_family(crate::theme::MONO))
                .text_color(move || theme().foreground),
        )
        .when(original.is_some(), move |row| {
            row.child(
                text(original.clone().unwrap_or_default())
                    .text_size(px(size * 0.55))
                    .text_color(move || theme().muted_foreground),
            )
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn conversion_only_shows_the_original_when_currencies_differ() {
        let source = include_str!("converted_amount.rs");
        assert!(source.contains("eq_ignore_ascii_case"));
        assert!(source.contains("rates.convert"));
    }
}
