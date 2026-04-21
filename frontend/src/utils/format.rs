use rust_decimal::Decimal;

/// Format an amount as currency string (e.g., "€1,500.00", "$9.99", "¥1,500")
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
    let no_decimals = currency == "JPY";
    let formatted = format_decimal(abs, no_decimals);

    if amount.is_sign_negative() {
        format!("-{}{}", symbol, formatted)
    } else {
        format!("{}{}", symbol, formatted)
    }
}

fn format_decimal(d: Decimal, no_decimals: bool) -> String {
    let s = format!("{}", d);
    let (integer_part, decimal_part) = if let Some(dot_pos) = s.find('.') {
        (&s[..dot_pos], Some(&s[dot_pos + 1..]))
    } else {
        (s.as_str(), None)
    };

    let integer_with_commas = add_thousands_separators(integer_part);

    if no_decimals {
        integer_with_commas
    } else {
        let decimal = match decimal_part {
            Some(dp) => format!("{:0<2}", dp),
            None => "00".to_string(),
        };
        let decimal = &decimal[..2];
        format!("{}.{}", integer_with_commas, decimal)
    }
}

/// Add thousands separators (commas) to an integer string.
/// e.g. "1500" -> "1,500", "0" -> "0"
fn add_thousands_separators(s: &str) -> String {
    let mut result = String::new();
    let digits: Vec<char> = s.chars().collect();
    let len = digits.len();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }
    result
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
        assert_eq!(format_currency(dec!(1500), "JPY"), "¥1,500");
    }

    #[test]
    fn format_jpy_no_decimals() {
        assert_eq!(format_currency(dec!(12345), "JPY"), "¥12,345");
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
    fn format_negative_thousands() {
        assert_eq!(format_currency(dec!(-1500), "EUR"), "-€1,500.00");
    }

    #[test]
    fn format_whole_number() {
        assert_eq!(format_currency(dec!(10), "EUR"), "€10.00");
    }

    #[test]
    fn format_thousands() {
        assert_eq!(format_currency(dec!(1500), "EUR"), "€1,500.00");
    }

    #[test]
    fn format_millions() {
        assert_eq!(format_currency(dec!(1234567.89), "USD"), "$1,234,567.89");
    }

    #[test]
    fn get_symbol_eur() {
        assert_eq!(get_currency_symbol("EUR"), "€");
    }

    #[test]
    fn get_symbol_usd() {
        assert_eq!(get_currency_symbol("USD"), "$");
    }

    // ── Additional format tests ──
    #[test]
    fn format_cad() {
        assert_eq!(format_currency(dec!(10), "CAD"), "$10.00");
    }

    #[test]
    fn format_aud() {
        assert_eq!(format_currency(dec!(10), "AUD"), "$10.00");
    }

    #[test]
    fn format_small_decimal() {
        assert_eq!(format_currency(dec!(0.01), "EUR"), "€0.01");
    }

    #[test]
    fn format_large_thousands() {
        assert_eq!(format_currency(dec!(100000), "USD"), "$100,000.00");
    }

    #[test]
    fn get_symbol_gbp() {
        assert_eq!(get_currency_symbol("GBP"), "£");
    }

    #[test]
    fn get_symbol_jpy() {
        assert_eq!(get_currency_symbol("JPY"), "¥");
    }

    #[test]
    fn get_symbol_cad() {
        assert_eq!(get_currency_symbol("CAD"), "$");
    }

    #[test]
    fn get_symbol_aud() {
        assert_eq!(get_currency_symbol("AUD"), "$");
    }

    #[test]
    fn get_symbol_unknown() {
        assert_eq!(get_currency_symbol("BRL"), "?");
    }

    #[test]
    fn format_decimal_no_fraction() {
        assert_eq!(format_currency(dec!(100), "EUR"), "€100.00");
    }

    #[test]
    fn add_thousands_separators_single_digit() {
        assert_eq!(add_thousands_separators("5"), "5");
    }

    #[test]
    fn add_thousands_separators_two_digits() {
        assert_eq!(add_thousands_separators("50"), "50");
    }

    #[test]
    fn add_thousands_separators_three_digits() {
        assert_eq!(add_thousands_separators("500"), "500");
    }

    #[test]
    fn add_thousands_separators_four_digits() {
        assert_eq!(add_thousands_separators("5000"), "5,000");
    }

    #[test]
    fn add_thousands_separators_seven_digits() {
        assert_eq!(add_thousands_separators("5000000"), "5,000,000");
    }

    #[test]
    fn format_decimal_no_decimals_jpy() {
        assert_eq!(format_currency(dec!(1234), "JPY"), "¥1,234");
    }
}
