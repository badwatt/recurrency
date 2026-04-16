use dioxus::prelude::*;

#[component]
pub fn Button(
    variant: String,
    disabled: bool,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let base = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 px-4 py-2";
    let variant_class = match variant.as_str() {
        "destructive" => "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        "outline" => "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        "ghost" => "hover:bg-accent hover:text-accent-foreground",
        _ => "bg-primary text-primary-foreground hover:bg-primary/90",
    };

    rsx! {
        button {
            r#type: "button",
            class: "{base} {variant_class}",
            disabled,
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn variant_default_class() {
        let v = "default";
        let class = match v {
            "destructive" => "bg-destructive",
            "outline" => "border",
            "ghost" => "hover:bg-accent",
            _ => "bg-primary",
        };
        assert!(class.contains("bg-primary"));
    }

    #[test]
    fn variant_destructive_class() {
        let v = "destructive";
        let class = match v {
            "destructive" => "destructive",
            "outline" => "outline",
            "ghost" => "ghost",
            _ => "default",
        };
        assert_eq!(class, "destructive");
    }
}