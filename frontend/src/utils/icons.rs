use dioxus::prelude::*;
use lucide_dioxus::{
    Activity, Baby, Banknote, Bell, Bike, Book, Box, Building, Bus, Cake, Camera, Compass, Cookie, Crown, DollarSign, Droplet, Flame, Globe, Hand, Headphones, Key, Landmark, Leaf, Lightbulb, MapPin, Mic, Mountain, Package, Palette, Percent, Phone, PiggyBank, Pizza, Printer, Rocket, Scissors, Shirt, ShoppingCart, Sparkles, Ticket, Trophy, Tv, Umbrella, Video, WavesHorizontal, Wrench, Beer, Briefcase, Car, Cloud, CloudRain, Coffee, CreditCard, Database,
    Film, Flower, Gamepad2, Gift, GraduationCap, Heart, House, Laptop, Lock, Monitor, Moon,
    EighthNotes, Pill, Plane, Server, Shield, ShoppingBag, Smartphone, Star, Stethoscope, Sun,
    Tag, Thermometer, TreePine, Utensils, Wallet, Wifi, Wine, Zap,
};

/// Available icon names (matching lucide-react icon names from original).
pub const ICON_NAMES: &[&str] = &[
    "House",
    "ShoppingBag",
    "Coffee",
    "Car",
    "Heart",
    "Film",
    "Music",
    "Gamepad2",
    "Briefcase",
    "GraduationCap",
    "Plane",
    "Zap",
    "Wifi",
    "Smartphone",
    "Laptop",
    "Monitor",
    "Cloud",
    "Database",
    "Server",
    "Lock",
    "Shield",
    "CreditCard",
    "Wallet",
    "Gift",
    "Star",
    "Sun",
    "Moon",
    "CloudRain",
    "Thermometer",
    "Activity",
    "Pill",
    "Stethoscope",
    "Baby",
    "Flower",
    "TreePine",
    "Utensils",
    "Beer",
    "Wine",
    "Tag",
    "Banknote",
    "Bell",
    "Bike",
    "Book",
    "Box",
    "Building2",
    "Bus",
    "Cake",
    "Camera",
    "Compass",
    "Cookie",
    "Crown",
    "DollarSign",
    "Droplet",
    "Flame",
    "Globe",
    "Hand",
    "Headphones",
    "Key",
    "Landmark",
    "Leaf",
    "Lightbulb",
    "MapPin",
    "Mic",
    "Mountain",
    "Package",
    "Palette",
    "Percent",
    "Phone",
    "PiggyBank",
    "Pizza",
    "Printer",
    "Rocket",
    "Scissors",
    "Shirt",
    "ShoppingCart",
    "Sparkles",
    "Ticket",
    "Trophy",
    "Tv",
    "Umbrella",
    "Video",
    "Waves",
    "Wrench",
];

/// Default icon when name is unknown or null.
pub const DEFAULT_ICON: &str = "Tag";

/// Map icon name to a lucide Element.
pub fn icon_display(name: &str) -> Element {
    match name {
        "House" => rsx! { House { size: 16 } },
        "ShoppingBag" => rsx! { ShoppingBag { size: 16 } },
        "Coffee" => rsx! { Coffee { size: 16 } },
        "Car" => rsx! { Car { size: 16 } },
        "Heart" => rsx! { Heart { size: 16 } },
        "Film" => rsx! { Film { size: 16 } },
        "Music" => rsx! { EighthNotes { size: 16 } },
        "Gamepad2" => rsx! { Gamepad2 { size: 16 } },
        "Briefcase" => rsx! { Briefcase { size: 16 } },
        "GraduationCap" => rsx! { GraduationCap { size: 16 } },
        "Plane" => rsx! { Plane { size: 16 } },
        "Zap" => rsx! { Zap { size: 16 } },
        "Wifi" => rsx! { Wifi { size: 16 } },
        "Smartphone" => rsx! { Smartphone { size: 16 } },
        "Laptop" => rsx! { Laptop { size: 16 } },
        "Monitor" => rsx! { Monitor { size: 16 } },
        "Cloud" => rsx! { Cloud { size: 16 } },
        "Database" => rsx! { Database { size: 16 } },
        "Server" => rsx! { Server { size: 16 } },
        "Lock" => rsx! { Lock { size: 16 } },
        "Shield" => rsx! { Shield { size: 16 } },
        "CreditCard" => rsx! { CreditCard { size: 16 } },
        "Wallet" => rsx! { Wallet { size: 16 } },
        "Gift" => rsx! { Gift { size: 16 } },
        "Star" => rsx! { Star { size: 16 } },
        "Sun" => rsx! { Sun { size: 16 } },
        "Moon" => rsx! { Moon { size: 16 } },
        "CloudRain" => rsx! { CloudRain { size: 16 } },
        "Thermometer" => rsx! { Thermometer { size: 16 } },
        "Activity" => rsx! { Activity { size: 16 } },
        "Pill" => rsx! { Pill { size: 16 } },
        "Stethoscope" => rsx! { Stethoscope { size: 16 } },
        "Baby" => rsx! { Baby { size: 16 } },
        "Flower" => rsx! { Flower { size: 16 } },
        "TreePine" => rsx! { TreePine { size: 16 } },
        "Utensils" => rsx! { Utensils { size: 16 } },
        "Beer" => rsx! { Beer { size: 16 } },
        "Wine" => rsx! { Wine { size: 16 } },
        "Banknote" => rsx! { Banknote { size: 16 } },
        "Bell" => rsx! { Bell { size: 16 } },
        "Bike" => rsx! { Bike { size: 16 } },
        "Book" => rsx! { Book { size: 16 } },
        "Box" => rsx! { Box { size: 16 } },
        "Building2" => rsx! { Building { size: 16 } },
        "Bus" => rsx! { Bus { size: 16 } },
        "Cake" => rsx! { Cake { size: 16 } },
        "Camera" => rsx! { Camera { size: 16 } },
        "Compass" => rsx! { Compass { size: 16 } },
        "Cookie" => rsx! { Cookie { size: 16 } },
        "Crown" => rsx! { Crown { size: 16 } },
        "DollarSign" => rsx! { DollarSign { size: 16 } },
        "Droplet" => rsx! { Droplet { size: 16 } },
        "Flame" => rsx! { Flame { size: 16 } },
        "Globe" => rsx! { Globe { size: 16 } },
        "Hand" => rsx! { Hand { size: 16 } },
        "Headphones" => rsx! { Headphones { size: 16 } },
        "Key" => rsx! { Key { size: 16 } },
        "Landmark" => rsx! { Landmark { size: 16 } },
        "Leaf" => rsx! { Leaf { size: 16 } },
        "Lightbulb" => rsx! { Lightbulb { size: 16 } },
        "MapPin" => rsx! { MapPin { size: 16 } },
        "Mic" => rsx! { Mic { size: 16 } },
        "Mountain" => rsx! { Mountain { size: 16 } },
        "Package" => rsx! { Package { size: 16 } },
        "Palette" => rsx! { Palette { size: 16 } },
        "Percent" => rsx! { Percent { size: 16 } },
        "Phone" => rsx! { Phone { size: 16 } },
        "PiggyBank" => rsx! { PiggyBank { size: 16 } },
        "Pizza" => rsx! { Pizza { size: 16 } },
        "Printer" => rsx! { Printer { size: 16 } },
        "Rocket" => rsx! { Rocket { size: 16 } },
        "Scissors" => rsx! { Scissors { size: 16 } },
        "Shirt" => rsx! { Shirt { size: 16 } },
        "ShoppingCart" => rsx! { ShoppingCart { size: 16 } },
        "Sparkles" => rsx! { Sparkles { size: 16 } },
        "Ticket" => rsx! { Ticket { size: 16 } },
        "Trophy" => rsx! { Trophy { size: 16 } },
        "Tv" => rsx! { Tv { size: 16 } },
        "Umbrella" => rsx! { Umbrella { size: 16 } },
        "Video" => rsx! { Video { size: 16 } },
        "Waves" => rsx! { WavesHorizontal { size: 16 } },
        "Wrench" => rsx! { Wrench { size: 16 } },
        "Tag" => rsx! { Tag { size: 16 } },
        _ => rsx! { Tag { size: 16 } },
    }
}

/// Convert a kebab-case icon name from the backend to PascalCase.
/// e.g., "tree-pine" → "TreePine"
pub fn normalize_icon_name(name: &str) -> String {
    if name.is_empty() {
        return DEFAULT_ICON.to_string();
    }
    name.split('-')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Check if an icon name is available.
pub fn is_valid_icon(name: &str) -> bool {
    let normalized = normalize_icon_name(name);
    ICON_NAMES.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_kebab_case() {
        assert_eq!(normalize_icon_name("tree-pine"), "TreePine");
        assert_eq!(normalize_icon_name("shopping-bag"), "ShoppingBag");
        assert_eq!(normalize_icon_name("gamepad-2"), "Gamepad2");
    }

    #[test]
    fn normalize_single_word() {
        assert_eq!(normalize_icon_name("house"), "House");
        assert_eq!(normalize_icon_name("film"), "Film");
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize_icon_name(""), "Tag");
    }

    #[test]
    fn valid_icons() {
        assert!(is_valid_icon("house"));
        assert!(is_valid_icon("tree-pine"));
        assert!(is_valid_icon("Gamepad2"));
    }

    #[test]
    fn invalid_icon_falls_to_default() {
        assert!(!is_valid_icon("nonexistent"));
        assert_eq!(normalize_icon_name("nonexistent"), "Nonexistent");
    }

    #[test]
    fn icon_count() {
        assert_eq!(ICON_NAMES.len(), 83);
    }

    #[test]
    fn default_icon_is_tag() {
        assert_eq!(DEFAULT_ICON, "Tag");
    }

    #[test]
    fn normalize_already_pascal() {
        assert_eq!(normalize_icon_name("House"), "House");
        assert_eq!(normalize_icon_name("Film"), "Film");
    }

    #[test]
    fn is_valid_icon_direct() {
        assert!(is_valid_icon("House"));
        assert!(is_valid_icon("Tag"));
    }

    #[test]
    fn is_valid_icon_from_kebab() {
        assert!(is_valid_icon("shopping-bag"));
    }

    #[test]
    fn is_valid_icon_empty_false() {
        assert!(is_valid_icon(""));
    }

    #[test]
    fn icon_names_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for name in ICON_NAMES {
            assert!(seen.insert(*name), "Duplicate icon name: {}", name);
        }
    }

    #[test]
    fn normalize_multiple_hyphens() {
        assert_eq!(normalize_icon_name("a-b-c"), "ABC");
    }

    #[test]
    fn normalize_single_char_parts() {
        assert_eq!(normalize_icon_name("a-b"), "AB");
    }

    #[test]
    fn normalize_preserves_numbers() {
        assert_eq!(normalize_icon_name("gamepad-2"), "Gamepad2");
    }

    #[test]
    fn icon_display_known_returns_element() {
        assert!(icon_display("House").is_ok());
        assert!(icon_display("Film").is_ok());
        assert!(icon_display("Tag").is_ok());
    }

    #[test]
    fn icon_display_unknown_returns_element() {
        assert!(icon_display("NonExistent").is_ok());
    }
}
