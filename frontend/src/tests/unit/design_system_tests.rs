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
    // Fonts are self-hosted via Dioxus assets (frontend/assets/fonts/).
    // No Google Fonts @import — eliminates external CDN dependency.
    let fonts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    assert!(fonts_dir.exists(), "assets/fonts directory must exist");
    let woff2_count = std::fs::read_dir(&fonts_dir)
        .map(|entries| entries.filter(|e| e.as_ref().unwrap().path().extension().map_or(false, |ext| ext == "woff2")).count())
        .unwrap_or(0);
    assert!(woff2_count >= 2, "Must have at least 2 woff2 font files, found {}", woff2_count);
}

#[test]
fn cabinet_grotesk_in_app_inline_style() {
    let app_rs = include_str!("../../lib.rs");
    assert!(
        app_rs.contains("Cabinet Grotesk") || app_rs.contains("Cabinet+Grotesk"),
        "app inline style must reference Cabinet Grotesk font"
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
