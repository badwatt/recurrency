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
        if i > 0 && (len - i) % 3 == 0 {
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
}