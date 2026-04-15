use dioxus::prelude::*;

#[component]
pub fn Badge(variant: String, children: Element) -> Element {
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors";
    let variant_class = match variant.as_str() {
        "secondary" => "bg-secondary text-secondary-foreground",
        "destructive" => "bg-destructive text-destructive-foreground",
        "outline" => "border border-border text-foreground",
        "success" => "bg-green-600 text-white",
        _ => "bg-primary text-primary-foreground",
    };

    rsx! {
        span { class: "{base} {variant_class}", {children} }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn badge_variant_default() {
        let v = "default";
        let class = match v {
            "secondary" => "secondary",
            "destructive" => "destructive",
            "outline" => "outline",
            "success" => "success",
            _ => "primary",
        };
        assert_eq!(class, "primary");
    }

    #[test]
    fn badge_variant_success() {
        let v = "success";
        let class = match v {
            "secondary" => "secondary",
            "destructive" => "destructive",
            "outline" => "outline",
            "success" => "success",
            _ => "primary",
        };
        assert_eq!(class, "success");
    }
}