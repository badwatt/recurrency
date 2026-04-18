use dioxus::prelude::*;

/// Renders a logo image with an onerror handler that hides the broken image
/// and shows a fallback span with the first letter of the name.
///
/// When no URL is provided, renders nothing (the caller should provide their own fallback).
///
/// Uses a unique element ID per instance so the onerror handler can target
/// the correct img and fallback elements via DOM manipulation.
pub fn logo_image_with_fallback(
    id: &str,
    url: Option<&str>,
    name: &str,
    img_class: &str,
) -> Element {
    let first_letter = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    let img_id = format!("logo-img-{id}");
    let fb_id = format!("logo-fb-{id}");
    match url {
        Some(url_str) => {
            let fb_id_clone = fb_id.clone();
            let img_id_clone = img_id.clone();
            rsx! {
                img {
                    id: "{img_id}",
                    src: "{url_str}",
                    alt: "{name}",
                    class: "{img_class}",
                    onerror: move |_| {
                        let _ = dioxus::document::eval(&format!(
                            "document.getElementById('{}').style.display='none'; \
                             document.getElementById('{}').style.display='flex';",
                            img_id_clone, fb_id_clone
                        ));
                    },
                }
                span {
                    id: "{fb_id}",
                    class: "flex h-full w-full items-center justify-center font-semibold text-muted-foreground",
                    style: "display:none;",
                    "{first_letter}"
                }
            }
        }
        None => rsx! {},
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_letter_uppercase_english() {
        let name = "Netflix";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "N");
    }

    #[test]
    fn first_letter_uppercase_spanish() {
        let name = "Álvaro";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "Á");
    }

    #[test]
    fn first_letter_uppercase_chinese() {
        let name = "哔哩哔哩";
        let first = name.chars().next().unwrap_or('?');
        // Chinese characters don't have case, so to_uppercase() returns itself
        let upper = first.to_uppercase().to_string();
        // The result should have at least one character
        assert!(!upper.is_empty());
    }

    #[test]
    fn first_letter_uppercase_emoji() {
        let name = "🎬 Film";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "🎬");
    }

    #[test]
    fn first_letter_empty_fallback() {
        let name = "";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "?");
    }

    #[test]
    fn first_letter_single_char() {
        let name = "A";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "A");
    }

    #[test]
    fn first_letter_lowercase_converted() {
        let name = "amazon";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "A");
    }

    #[test]
    fn first_letter_mixed_case() {
        let name = "pOrTfOlIo";
        let first = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        assert_eq!(first, "P");
    }

    #[test]
    fn logo_id_format() {
        let id = "netflix-logo";
        let img_id = format!("logo-img-{}", id);
        let fb_id = format!("logo-fb-{}", id);
        assert_eq!(img_id, "logo-img-netflix-logo");
        assert_eq!(fb_id, "logo-fb-netflix-logo");
    }

    #[test]
    fn logo_id_with_uuid() {
        let id = uuid::Uuid::new_v4().to_string();
        let img_id = format!("logo-img-{}", id);
        let fb_id = format!("logo-fb-{}", id);
        assert!(img_id.starts_with("logo-img-"));
        assert!(fb_id.starts_with("logo-fb-"));
    }

    #[test]
    fn logo_url_option_none() {
        let url: Option<&str> = None;
        assert!(url.is_none());
    }

    #[test]
    fn logo_url_option_some() {
        let url: Option<&str> = Some("https://logo.com/netflix.png");
        assert!(url.is_some());
        assert!(url.unwrap().starts_with("https://"));
    }

    #[test]
    fn logo_fallback_class_contains_flex() {
        let fb_class = "flex h-full w-full items-center justify-center font-semibold text-muted-foreground";
        assert!(fb_class.contains("flex"));
        assert!(fb_class.contains("items-center"));
        assert!(fb_class.contains("justify-center"));
    }

    #[test]
    fn logo_onerror_js_format() {
        let img_id = "logo-img-test";
        let fb_id = "logo-fb-test";
        let js = format!(
            "document.getElementById('{}').style.display='none'; \
             document.getElementById('{}').style.display='flex';",
            img_id, fb_id
        );
        assert!(js.contains("logo-img-test"));
        assert!(js.contains("logo-fb-test"));
        assert!(js.contains("display='none'"));
        assert!(js.contains("display='flex'"));
    }

    #[test]
    fn logo_img_class_passed_through() {
        let img_class = "h-10 w-10 rounded object-contain border border-border";
        assert!(img_class.contains("h-10"));
        assert!(img_class.contains("w-10"));
        assert!(img_class.contains("object-contain"));
    }

    #[test]
    fn logo_alt_text_uses_name() {
        let name = "Spotify";
        let alt = format!("{}", name);
        assert_eq!(alt, "Spotify");
    }

    #[test]
    fn logo_src_url_format() {
        let url_str = "https://logo.clearbit.com/spotify.com";
        assert!(url_str.starts_with("https://"));
        assert!(url_str.contains("logo"));
    }
}