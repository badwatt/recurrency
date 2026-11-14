//! Renders an amount in the user's currency, with the original currency lighter.

use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::state::rates::RATES;

#[component]
pub fn ConvertedAmount(amount: Decimal, currency: String) -> Element {
    let (main, original) = RATES().display_amount_parts(amount, &currency);
    rsx! {
        span { class: "amount-sensitive", {main.to_string()} }
        if let Some(original) = original {
            span { class: "amount-sensitive font-light text-muted-foreground/70", " ({original})" }
        }
    }
}
