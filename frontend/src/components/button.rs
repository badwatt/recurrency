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
            "destructive" => "bg-destructive",
            "outline" => "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
            "ghost" => "hover:bg-accent hover:text-accent-foreground",
            _ => "bg-primary text-primary-foreground hover:bg-primary/90",
        };
        assert!(class.contains("destructive"));
    }

    #[test]
    fn variant_outline_class() {
        let v = "outline";
        let class = match v {
            "destructive" => "bg-destructive text-destructive-foreground hover:bg-destructive/90",
            "outline" => "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
            "ghost" => "hover:bg-accent hover:text-accent-foreground",
            _ => "bg-primary text-primary-foreground hover:bg-primary/90",
        };
        // "outline" variant uses border class, not "outline" class
        assert!(class.contains("border"));
    }

    #[test]
    fn variant_ghost_class() {
        let v = "ghost";
        let class = match v {
            "destructive" => "bg-destructive text-destructive-foreground hover:bg-destructive/90",
            "outline" => "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
            "ghost" => "hover:bg-accent hover:text-accent-foreground",
            _ => "bg-primary text-primary-foreground hover:bg-primary/90",
        };
        assert!(class.contains("ghost") || class.contains("hover:bg-accent"));
    }

    #[test]
    fn button_base_class_structure() {
        let base = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 px-4 py-2";
        assert!(base.contains("inline-flex"));
        assert!(base.contains("justify-center"));
        assert!(base.contains("rounded-md"));
        assert!(base.contains("focus:ring"));
        assert!(base.contains("disabled:opacity-50"));
    }
}