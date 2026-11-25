/// Supported ISO 4217 currencies with display labels.
pub const CURRENCIES: &[(&str, &str)] = &[
    ("EUR", "EUR — Euro"),
    ("AED", "AED — UAE Dirham"),
    ("AFN", "AFN — Afghani"),
    ("ALL", "ALL — Lek"),
    ("AMD", "AMD — Armenian Dram"),
    ("AOA", "AOA — Kwanza"),
    ("ARS", "ARS — Argentine Peso"),
    ("AUD", "AUD — Australian Dollar"),
    ("AWG", "AWG — Aruban Florin"),
    ("AZN", "AZN — Azerbaijan Manat"),
    ("BAM", "BAM — Convertible Mark"),
    ("BBD", "BBD — Barbados Dollar"),
    ("BDT", "BDT — Taka"),
    ("BHD", "BHD — Bahraini Dinar"),
    ("BIF", "BIF — Burundi Franc"),
    ("BMD", "BMD — Bermudian Dollar"),
    ("BND", "BND — Brunei Dollar"),
    ("BOB", "BOB — Boliviano"),
    ("BOV", "BOV — Mvdol"),
    ("BRL", "BRL — Brazilian Real"),
    ("BSD", "BSD — Bahamian Dollar"),
    ("BTN", "BTN — Ngultrum"),
    ("BWP", "BWP — Pula"),
    ("BYN", "BYN — Belarusian Ruble"),
    ("BZD", "BZD — Belize Dollar"),
    ("CAD", "CAD — Canadian Dollar"),
    ("CDF", "CDF — Congolese Franc"),
    ("CHE", "CHE — WIR Euro"),
    ("CHF", "CHF — Swiss Franc"),
    ("CHW", "CHW — WIR Franc"),
    ("CLF", "CLF — Unidad de Fomento"),
    ("CLP", "CLP — Chilean Peso"),
    ("CNY", "CNY — Yuan Renminbi"),
    ("COP", "COP — Colombian Peso"),
    ("COU", "COU — Unidad de Valor Real"),
    ("CRC", "CRC — Costa Rican Colon"),
    ("CUP", "CUP — Cuban Peso"),
    ("CVE", "CVE — Cabo Verde Escudo"),
    ("CZK", "CZK — Czech Koruna"),
    ("DJF", "DJF — Djibouti Franc"),
    ("DKK", "DKK — Danish Krone"),
    ("DOP", "DOP — Dominican Peso"),
    ("DZD", "DZD — Algerian Dinar"),
    ("EGP", "EGP — Egyptian Pound"),
    ("ERN", "ERN — Nakfa"),
    ("ETB", "ETB — Ethiopian Birr"),
    ("FJD", "FJD — Fiji Dollar"),
    ("FKP", "FKP — Falkland Islands Pound"),
    ("GBP", "GBP — Pound Sterling"),
    ("GEL", "GEL — Lari"),
    ("GHS", "GHS — Ghana Cedi"),
    ("GIP", "GIP — Gibraltar Pound"),
    ("GMD", "GMD — Dalasi"),
    ("GNF", "GNF — Guinean Franc"),
    ("GTQ", "GTQ — Quetzal"),
    ("GYD", "GYD — Guyana Dollar"),
    ("HKD", "HKD — Hong Kong Dollar"),
    ("HNL", "HNL — Lempira"),
    ("HTG", "HTG — Gourde"),
    ("HUF", "HUF — Forint"),
    ("IDR", "IDR — Rupiah"),
    ("ILS", "ILS — New Israeli Sheqel"),
    ("INR", "INR — Indian Rupee"),
    ("IQD", "IQD — Iraqi Dinar"),
    ("IRR", "IRR — Iranian Rial"),
    ("ISK", "ISK — Iceland Krona"),
    ("JMD", "JMD — Jamaican Dollar"),
    ("JOD", "JOD — Jordanian Dinar"),
    ("JPY", "JPY — Yen"),
    ("KES", "KES — Kenyan Shilling"),
    ("KGS", "KGS — Som"),
    ("KHR", "KHR — Riel"),
    ("KMF", "KMF — Comorian Franc "),
    ("KPW", "KPW — North Korean Won"),
    ("KRW", "KRW — Won"),
    ("KWD", "KWD — Kuwaiti Dinar"),
    ("KYD", "KYD — Cayman Islands Dollar"),
    ("KZT", "KZT — Tenge"),
    ("LAK", "LAK — Lao Kip"),
    ("LBP", "LBP — Lebanese Pound"),
    ("LKR", "LKR — Sri Lanka Rupee"),
    ("LRD", "LRD — Liberian Dollar"),
    ("LSL", "LSL — Loti"),
    ("LYD", "LYD — Libyan Dinar"),
    ("MAD", "MAD — Moroccan Dirham"),
    ("MDL", "MDL — Moldovan Leu"),
    ("MGA", "MGA — Malagasy Ariary"),
    ("MKD", "MKD — Denar"),
    ("MMK", "MMK — Kyat"),
    ("MNT", "MNT — Tugrik"),
    ("MOP", "MOP — Pataca"),
    ("MRU", "MRU — Ouguiya"),
    ("MUR", "MUR — Mauritius Rupee"),
    ("MVR", "MVR — Rufiyaa"),
    ("MWK", "MWK — Malawi Kwacha"),
    ("MXN", "MXN — Mexican Peso"),
    ("MXV", "MXV — Mexican Unidad de Inversion (UDI)"),
    ("MYR", "MYR — Malaysian Ringgit"),
    ("MZN", "MZN — Mozambique Metical"),
    ("NAD", "NAD — Namibia Dollar"),
    ("NGN", "NGN — Naira"),
    ("NIO", "NIO — Cordoba Oro"),
    ("NOK", "NOK — Norwegian Krone"),
    ("NPR", "NPR — Nepalese Rupee"),
    ("NZD", "NZD — New Zealand Dollar"),
    ("OMR", "OMR — Rial Omani"),
    ("PAB", "PAB — Balboa"),
    ("PEN", "PEN — Sol"),
    ("PGK", "PGK — Kina"),
    ("PHP", "PHP — Philippine Peso"),
    ("PKR", "PKR — Pakistan Rupee"),
    ("PLN", "PLN — Zloty"),
    ("PYG", "PYG — Guarani"),
    ("QAR", "QAR — Qatari Rial"),
    ("RON", "RON — Romanian Leu"),
    ("RSD", "RSD — Serbian Dinar"),
    ("RUB", "RUB — Russian Ruble"),
    ("RWF", "RWF — Rwanda Franc"),
    ("SAR", "SAR — Saudi Riyal"),
    ("SBD", "SBD — Solomon Islands Dollar"),
    ("SCR", "SCR — Seychelles Rupee"),
    ("SDG", "SDG — Sudanese Pound"),
    ("SEK", "SEK — Swedish Krona"),
    ("SGD", "SGD — Singapore Dollar"),
    ("SHP", "SHP — Saint Helena Pound"),
    ("SLE", "SLE — Leone"),
    ("SOS", "SOS — Somali Shilling"),
    ("SRD", "SRD — Surinam Dollar"),
    ("SSP", "SSP — South Sudanese Pound"),
    ("STN", "STN — Dobra"),
    ("SVC", "SVC — El Salvador Colon"),
    ("SYP", "SYP — Syrian Pound"),
    ("SZL", "SZL — Lilangeni"),
    ("THB", "THB — Baht"),
    ("TJS", "TJS — Somoni"),
    ("TMT", "TMT — Turkmenistan New Manat"),
    ("TND", "TND — Tunisian Dinar"),
    ("TOP", "TOP — Pa’anga"),
    ("TRY", "TRY — Turkish Lira"),
    ("TTD", "TTD — Trinidad and Tobago Dollar"),
    ("TWD", "TWD — New Taiwan Dollar"),
    ("TZS", "TZS — Tanzanian Shilling"),
    ("UAH", "UAH — Hryvnia"),
    ("UGX", "UGX — Uganda Shilling"),
    ("USD", "USD — US Dollar"),
    ("USN", "USN — US Dollar (Next day)"),
    ("UYI", "UYI — Uruguay Peso en Unidades Indexadas (UI)"),
    ("UYU", "UYU — Peso Uruguayo"),
    ("UYW", "UYW — Unidad Previsional"),
    ("UZS", "UZS — Uzbekistan Sum"),
    ("VED", "VED — Bolívar Soberano"),
    ("VES", "VES — Bolívar Soberano"),
    ("VND", "VND — Dong"),
    ("VUV", "VUV — Vatu"),
    ("WST", "WST — Tala"),
    ("XAD", "XAD — Arab Accounting Dinar"),
    ("XAF", "XAF — CFA Franc BEAC"),
    ("XAG", "XAG — Silver"),
    ("XAU", "XAU — Gold"),
    (
        "XBA",
        "XBA — Bond Markets Unit European Composite Unit (EURCO)",
    ),
    (
        "XBB",
        "XBB — Bond Markets Unit European Monetary Unit (E.M.U.-6)",
    ),
    (
        "XBC",
        "XBC — Bond Markets Unit European Unit of Account 9 (E.U.A.-9)",
    ),
    (
        "XBD",
        "XBD — Bond Markets Unit European Unit of Account 17 (E.U.A.-17)",
    ),
    ("XCD", "XCD — East Caribbean Dollar"),
    ("XCG", "XCG — Caribbean Guilder"),
    ("XDR", "XDR — SDR (Special Drawing Right)"),
    ("XOF", "XOF — CFA Franc BCEAO"),
    ("XPD", "XPD — Palladium"),
    ("XPF", "XPF — CFP Franc"),
    ("XPT", "XPT — Platinum"),
    ("XSU", "XSU — Sucre"),
    (
        "XTS",
        "XTS — Codes specifically reserved for testing purposes",
    ),
    ("XUA", "XUA — ADB Unit of Account"),
    (
        "XXX",
        "XXX — The codes assigned for transactions where no currency is involved",
    ),
    ("YER", "YER — Yemeni Rial"),
    ("ZAR", "ZAR — Rand"),
    ("ZMW", "ZMW — Zambian Kwacha"),
    ("ZWG", "ZWG — Zimbabwe Gold"),
];

/// Supported billing frequencies.
pub const FREQUENCIES: &[(&str, &str)] = &[
    ("daily", "Daily"),
    ("weekly", "Weekly"),
    ("monthly", "Monthly"),
    ("quarterly", "Quarterly"),
    ("yearly", "Yearly"),
];

/// Recurrence units for filters and summaries.
pub const RECURRENCE_UNITS: &[(&str, &str)] = &[
    ("day", "Days"),
    ("week", "Weeks"),
    ("month", "Months"),
    ("year", "Years"),
];

/// Debt types stored by the backend.
pub const DEBT_TYPE_MANUAL: &str = "manual";
pub const DEBT_TYPE_AUTOMATIC: &str = "automatic";

/// Summary notification periods.
pub const SUMMARY_PERIOD_CURRENT: &str = "current";
pub const SUMMARY_PERIOD_NEXT: &str = "next";

/// Highest valid day of a month for recurring schedules.
pub const MAX_MONTH_DAY: u8 = 31;

/// Default currency.
pub const DEFAULT_CURRENCY: &str = "EUR";

/// App version (workspace Cargo.toml is the single source).
#[allow(dead_code)]
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currencies_has_eur_first() {
        assert_eq!(CURRENCIES[0].0, "EUR");
    }

    #[test]
    fn currencies_count() {
        assert_eq!(CURRENCIES.len(), 178);
    }

    #[test]
    fn currencies_contains_all_expected() {
        let codes: Vec<&str> = CURRENCIES.iter().map(|(k, _)| *k).collect();
        assert!(codes.contains(&"EUR"));
        assert!(codes.contains(&"USD"));
        assert!(codes.contains(&"GBP"));
        assert!(codes.contains(&"JPY"));
        assert!(codes.contains(&"CAD"));
        assert!(codes.contains(&"AUD"));
        assert!(codes.contains(&"CLP"));
        assert!(codes.contains(&"MXN"));
    }

    #[test]
    fn currencies_labels_match_codes() {
        for (code, label) in CURRENCIES {
            assert!(label.contains(code));
        }
    }

    #[test]
    fn frequencies_count() {
        assert_eq!(FREQUENCIES.len(), 5);
    }

    #[test]
    fn frequencies_contains_all_expected() {
        let freqs: Vec<&str> = FREQUENCIES.iter().map(|(k, _)| *k).collect();
        assert!(freqs.contains(&"daily"));
        assert!(freqs.contains(&"weekly"));
        assert!(freqs.contains(&"monthly"));
        assert!(freqs.contains(&"quarterly"));
        assert!(freqs.contains(&"yearly"));
    }

    #[test]
    fn frequencies_labels_capitalized() {
        for (_, label) in FREQUENCIES {
            assert!(label.chars().next().unwrap().is_uppercase());
        }
    }

    #[test]
    fn default_currency_is_eur() {
        assert_eq!(DEFAULT_CURRENCY, "EUR");
    }

    #[test]
    fn currency_labels() {
        assert_eq!(CURRENCIES[0].1, "EUR — Euro");
        assert_eq!(
            CURRENCIES
                .iter()
                .find(|(code, _)| *code == "CLP")
                .unwrap()
                .1,
            "CLP — Chilean Peso"
        );
    }

    #[test]
    fn currency_codes_are_unique_and_sorted_after_eur() {
        let codes: Vec<&str> = CURRENCIES.iter().map(|(code, _)| *code).collect();
        assert_eq!(codes.len(), 178);
        let mut sorted = codes[1..].to_vec();
        sorted.sort_unstable();
        assert_eq!(codes[0], "EUR");
        assert_eq!(codes[1..], sorted);
    }

    #[test]
    fn app_version_is_set() {
        assert!(!APP_VERSION.is_empty());
    }
}
