//! Guards against page roots that transition every property on mount.

use std::path::Path;

/// Tailwind's `duration-*` utilities set `transition-duration` as well as
/// `animation-duration`. On a page root that also has `animate-in`, that
/// creates an implicit `transition: all` (the default transition-property)
/// and makes padding, borders and colours animate into place after the
/// loader disappears. Pages must animate with the plugin's default duration
/// or an explicit `[animation-duration:...]` override instead.
#[test]
fn page_roots_do_not_use_duration_with_animate_in() {
    let pages = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages");
    let mut offenders = Vec::new();

    fn visit(dir: &Path, offenders: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, offenders);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let Ok(source) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for (index, line) in source.lines().enumerate() {
                    if line.contains("animate-in") && line.contains("duration-") {
                        offenders.push(format!("{}:{}", path.display(), index + 1));
                    }
                }
            }
        }
    }

    visit(&pages, &mut offenders);
    assert!(
        offenders.is_empty(),
        "page roots combine animate-in with duration-*: {offenders:?}"
    );
}

#[test]
fn page_roots_use_opacity_only_entry() {
    let pages = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages");
    let mut offenders = Vec::new();

    fn visit(dir: &Path, offenders: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, offenders);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let Ok(source) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for (index, line) in source.lines().enumerate() {
                    if line.contains("animate-in fade-in") {
                        offenders.push(format!("{}:{}", path.display(), index + 1));
                    }
                }
            }
        }
    }

    visit(&pages, &mut offenders);
    assert!(
        offenders.is_empty(),
        "page roots must use the opacity-only page-enter animation: {offenders:?}"
    );
}

#[test]
fn focus_visible_outline_is_disabled_globally() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    assert!(
        css.contains(":focus-visible") && css.contains("outline: none;"),
        "the browser focus-visible outline must be disabled globally"
    );
}
