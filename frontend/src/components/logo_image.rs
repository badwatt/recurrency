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