/// Tests for radical redesign design tokens.

#[test]
fn cabinet_grotesk_is_sans_font_family() {
    let config = include_str!("../../../tailwind.config.js");
    assert!(
        config.contains("Cabinet Grotesk"),
        "tailwind.config.js must use Cabinet Grotesk as primary sans font"
    );
}

#[test]
fn font_import_in_tailwind_src() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    assert!(
        css.contains("Cabinet+Grotesk"),
        "tailwind.src.css must import Cabinet Grotesk from Google Fonts"
    );
}

#[test]
fn cabinet_grotesk_in_main_inline_style() {
    let main_rs = include_str!("../../main.rs");
    assert!(
        main_rs.contains("Cabinet Grotesk") || main_rs.contains("Cabinet+Grotesk"),
        "main.rs inline style must reference Cabinet Grotesk font"
    );
}

#[test]
fn warm_dark_background_in_css_vars() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    // Background should be a warm dark, not pure black or cool blue-black
    let dark_block = css.split(".dark").nth(1).unwrap_or("");
    assert!(
        dark_block.contains("--background"),
        "dark theme must define --background"
    );
    // The new warm dark should be around hsl(255, 15%, 5%) or similar
    // We verify it's not the old blue-black hsl(222.2, 84%, 4.9%)
    assert!(
        !dark_block.contains("222.2 84% 4.9%"),
        "dark background must not be the old blue-black (222.2 84% 4.9%)"
    );
}

#[test]
fn single_amber_accent_in_css_vars() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    // Primary should be an amber/warm gold, not blue
    assert!(
        css.contains("--primary:"),
        "must define --primary CSS variable"
    );
    // Old primary was 222.2 47.4% 11.2% (dark blue) — must change
    let light_block = css.split(":root").nth(1).unwrap_or("");
    assert!(
        !light_block.contains("222.2 47.4% 11.2%"),
        "primary must not be the old dark blue (222.2 47.4% 11.2%)"
    );
}

#[test]
fn design_system_uses_varied_border_radius() {
    let config = include_str!("../../../tailwind.config.js");
    assert!(
        config.contains("borderRadius"),
        "tailwind.config.js must define borderRadius scale"
    );
    // Must have at least 3 radius values (varied, not uniform)
    let radius_section = config.split("borderRadius").nth(1).unwrap_or("");
    let comma_count = radius_section.matches(',').count();
    assert!(
        comma_count >= 2,
        "borderRadius must have at least 3 values for varied radius (found {} commas)",
        comma_count
    );
}
