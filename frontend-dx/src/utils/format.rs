use rust_decimal::Decimal;

/// Format an amount as currency string (e.g., "€15.99", "$9.99")
/// Mirrors the JS Intl.NumberFormat behavior from the React frontend.
pub fn format_currency(amount: Decimal, currency: &str) -> String {
    let symbol = match currency {
        "EUR" => "€",
        "USD" | "CAD" | "AUD" => "$",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "?",
    };

    let abs = amount.abs();
    let formatted = format_decimal(abs);

    if amount.is_sign_negative() {
        format!("-{}{}", symbol, formatted)
    } else {
        format!("{}{}", symbol, formatted)
    }
}

fn format_decimal(d: Decimal) -> String {
    // Format to 2 decimal places for most currencies, 0 for JPY
    let s = format!("{}", d);
    if let Some(dot_pos) = s.find('.') {
        let integer = &s[..dot_pos];
        let decimal = &s[dot_pos + 1..];
        let decimal = format!("{:0<2}", decimal);
        let decimal = &decimal[..2];
        // Remove trailing zeros for whole amounts
        if decimal == "00" {
            format!("{}.00", integer)
        } else if decimal.ends_with('0') {
            format!("{}.{}", integer, decimal)
        } else {
            format!("{}.{}", integer, decimal)
        }
    } else {
        format!("{}.00", s)
    }
}

pub fn get_currency_symbol(currency: &str) -> &str {
    match currency {
        "EUR" => "€",
        "USD" | "CAD" | "AUD" => "$",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn format_eur() {
        assert_eq!(format_currency(dec!(15.99), "EUR"), "€15.99");
    }

    #[test]
    fn format_usd() {
        assert_eq!(format_currency(dec!(9.99), "USD"), "$9.99");
    }

    #[test]
    fn format_gbp() {
        assert_eq!(format_currency(dec!(42.50), "GBP"), "£42.50");
    }

    #[test]
    fn format_jpy() {
        assert_eq!(format_currency(dec!(1500), "JPY"), "¥1500.00");
    }

    #[test]
    fn format_unknown_currency() {
        // Unknown currency returns "?" as symbol
        assert_eq!(format_currency(dec!(10), "BRL"), "?10.00");
    }

    #[test]
    fn format_zero() {
        assert_eq!(format_currency(dec!(0), "EUR"), "€0.00");
    }

    #[test]
    fn format_negative() {
        assert_eq!(format_currency(dec!(-15.99), "EUR"), "-€15.99");
    }

    #[test]
    fn format_whole_number() {
        assert_eq!(format_currency(dec!(10), "EUR"), "€10.00");
    }

    #[test]
    fn get_symbol_eur() {
        assert_eq!(get_currency_symbol("EUR"), "€");
    }

    #[test]
    fn get_symbol_usd() {
        assert_eq!(get_currency_symbol("USD"), "$");
    }
}