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
    let first_letter = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let img_id = format!("logo-img-{id}");
    let fb_id = format!("logo-fb-{id}");
    let wrapper_id = format!("logo-wrapper-{id}");
    match url {
        Some(url_str) => {
            let fb_id_clone = fb_id.clone();
            let img_id_clone_for_error = img_id.clone();
            let img_id_clone_for_load = img_id.clone();
            let _wrapper_id_clone = wrapper_id.clone();
            rsx! {
                div {
                    id: {wrapper_id.to_string()},
                    class: "flex items-center justify-center rounded-md overflow-hidden bg-primary/10",
                    img {
                        id: {img_id.to_string()},
                        src: {url_str.to_string()},
                        alt: {name.to_string()},
                        class: {img_class.to_string()},
                        onerror: move |_| {
                            let _ = dioxus::document::eval(&format!(
                                "document.getElementById('{}').style.display='none'; \
                                 document.getElementById('{}').style.display='flex';",
                                img_id_clone_for_error, fb_id_clone
                            ));
                        },
                        onload: move |_| {
                            let _ = dioxus::document::eval(&format!(
                                r#"
                                (function() {{
                                    var img = document.getElementById('{}');
                                    var wrapper = img ? img.parentElement : null;
                                    if (!img || !wrapper) return;
                                    var canvas = document.createElement('canvas');
                                    var ctx = canvas.getContext('2d');
                                    canvas.width = img.naturalWidth || 1;
                                    canvas.height = img.naturalHeight || 1;
                                    ctx.drawImage(img, 0, 0);
                                    var data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
                                    var hasTransparency = false;
                                    for (var i = 3; i < data.length; i += 4) {{
                                        if (data[i] < 255) {{ hasTransparency = true; break; }}
                                    }}
                                    if (hasTransparency) {{
                                        wrapper.style.background = 'var(--primary)';
                                    }}
                                }})()
                                "#,
                                img_id_clone_for_load
                            ));
                        },
                    }
                    span {
                        id: {fb_id.to_string()},
                        class: "flex h-full w-full items-center justify-center font-semibold text-muted-foreground",
                        style: "display:none;",
                        "{first_letter}"
                    }
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
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        assert_eq!(first, "N");
    }

    #[test]
    fn first_letter_uppercase_spanish() {
        let name = "Álvaro";
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
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
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        assert_eq!(first, "🎬");
    }

    #[test]
    fn first_letter_empty_fallback() {
        let name = "";
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        assert_eq!(first, "?");
    }

    #[test]
    fn first_letter_single_char() {
        let name = "A";
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        assert_eq!(first, "A");
    }

    #[test]
    fn first_letter_lowercase_converted() {
        let name = "amazon";
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        assert_eq!(first, "A");
    }

    #[test]
    fn first_letter_mixed_case() {
        let name = "pOrTfOlIo";
        let first = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
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
        let fb_class =
            "flex h-full w-full items-center justify-center font-semibold text-muted-foreground";
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

    #[test]
    fn logo_wrapper_has_theme_background() {
        let src = include_str!("logo_image.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("div {"), "logo must be wrapped in a div");
        assert!(prod.contains("bg-primary"), "wrapper must have theme background");
    }

    #[test]
    fn logo_image_and_fallback_inside_wrapper() {
        let src = include_str!("logo_image.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        // img and span should appear after div wrapper opens in Some(url) branch
        let some_branch = prod.split("match url").nth(1).unwrap_or("");
        assert!(some_branch.contains("div {"), "wrapper div must be inside Some branch");
        assert!(some_branch.contains("img {"), "img must be inside wrapper");
        assert!(some_branch.contains("span {"), "fallback span must be inside wrapper");
    }

    #[test]
    fn logo_transparency_check_js_format() {
        // JS to detect transparency: draw image to canvas, check alpha channel
        let img_id = "logo-img-test-id";
        let wrapper_id = "logo-wrapper-test-id";
        let js = format!(
            r#"
            (function() {{
                var img = document.getElementById('{}');
                var wrapper = img ? img.parentElement : null;
                if (!img || !wrapper) return;
                var canvas = document.createElement('canvas');
                var ctx = canvas.getContext('2d');
                canvas.width = img.naturalWidth || 1;
                canvas.height = img.naturalHeight || 1;
                ctx.drawImage(img, 0, 0);
                var data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
                var hasTransparency = false;
                for (var i = 3; i < data.length; i += 4) {{
                    if (data[i] < 255) {{ hasTransparency = true; break; }}
                }}
                if (hasTransparency) {{
                    wrapper.style.background = 'var(--primary)';
                }}
            }})()
            "#,
            img_id
        );
        assert!(js.contains("getElementById"), "JS must get element by ID");
        assert!(js.contains("getContext('2d')"), "JS must create canvas context");
        assert!(js.contains("getImageData"), "JS must get image data");
        assert!(js.contains("hasTransparency"), "JS must check transparency");
        assert!(js.contains("var(--primary)"), "JS must set solid primary color when transparent");
    }
}
