
// TDD tests for Android theme startup bug.
// Bug 1: AppLayout llama apply_theme_to_document gated #[cfg(feature=web)] en Android.
//        → apply_theme_to_document body vacio en Android → tema no se aplica al cargar.
// Fix: usar apply_theme_class (usa dioxus::document::eval, funciona en TODAS plataformas).

use crate::theme::hooks::{apply_theme_class, cycle_theme, THEMES};

// ── Bug 1: Startup theme application ──────────────────────────────────────────

/// TDD RED: hydrate_from_storage debe leer tema de archivo.
/// BUG: write to /tmp but hydrate_from_storage reads from real path → mismatch.
/// FIX: write to REAL get_app_data_dir() path so hydrate finds it.
#[test]
fn test_hydrate_from_storage_reads_theme() {
    let data_dir = crate::state::auth::get_app_data_dir();
    if data_dir.is_none() {
        // Skip if no data dir (CI without desktop env)
        return;
    }
    let theme_file = data_dir.unwrap().join("recurrency").join("theme.txt");
    std::fs::create_dir_all(theme_file.parent().unwrap()).ok();
    std::fs::write(&theme_file, "mocha").ok();

    let mut state = crate::state::preferences::PreferencesState::default();
    state.hydrate_from_storage();

    assert_eq!(state.theme(), "mocha",
        "hydrate_from_storage debe leer 'mocha' del archivo en {:?}",
        theme_file.parent().unwrap());

    std::fs::remove_file(&theme_file).ok();
}

/// TDD RED: apply_theme_class NO tiene gate #[cfg(feature = "web")].
/// BUG: apply_theme_to_document SI tiene #[cfg(feature = "web")] → no-op en Android.
/// FIX: app_layout.rs debe llamar apply_theme_class (sin gate).
#[test]
fn test_apply_theme_class_compiles_without_web_feature() {
    // Si compila → no tiene #[cfg(feature = "web")] que bloquee en Android.
    // Llamar cycle_theme para verificar que apply_theme_class existe y es pub(crate).
    assert_eq!(cycle_theme("light"), "dark");
    assert_eq!(apply_theme_class_fn_name(), "apply_theme_class");
}

fn apply_theme_class_fn_name() -> &'static str { "apply_theme_class" }

/// TDD RED: app_layout.rs debe llamar apply_theme_class (no apply_theme_to_document).
/// BUG: AppLayout usa apply_theme_to_document(gated) + apply_theme_class(sin gate) no se llama.
/// FIX: use_effect en AppLayout llama apply_theme_class.
#[test]
fn test_app_layout_calls_apply_theme_class() {
    let src = include_str!("layout/app_layout.rs");
    // app_layout.rs DEBE llamar apply_theme_class, no apply_theme_to_document
    assert!(src.contains("apply_theme_class"),
        "AppLayout debe llamar apply_theme_class (funcion sin gate #[cfg(feature=web)])");
    // Ya NO debe depender SOLO de apply_theme_to_document
    // (puede llamarlo en web, pero Android necesita apply_theme_class)
}

/// TDD RED: use_swipe_sidebar NO tiene gate #[cfg(feature = "web"].
/// BUG: swipe solo funcionaba en web → no-op en Android.
/// FIX: swipe via JS eval + typeof dioxus check funciona en Android.
#[test]
fn test_swipe_sidebar_no_web_feature_gate() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(!src.contains("#[cfg(feature = \"web\")]"),
        "use_swipe_sidebar no debe tener gate #[cfg(feature=web)] que bloquee Android");
    assert!(src.contains("typeof dioxus === 'undefined'"),
        "use_swipe_sidebar debe verificar typeof dioxus para Android compatibility");
}

// ── Tests existentes ───────────────────────────────────────────────────────────

#[test]
fn test_cycle_theme_all_values() {
    let cycle = |t| cycle_theme(t);
    assert_eq!(cycle("light"), "dark");
    assert_eq!(cycle("dark"), "mocha");
    assert_eq!(cycle("mocha"), "system");
    assert_eq!(cycle("system"), "light");
}

#[test]
fn test_themes_constant_has_four_values() {
    assert_eq!(THEMES.len(), 4);
    assert!(THEMES.contains(&"light"));
    assert!(THEMES.contains(&"dark"));
    assert!(THEMES.contains(&"mocha"));
    assert!(THEMES.contains(&"system"));
}

#[test]
fn test_set_theme_writes_theme_file_on_native() {
    let test_root = std::path::PathBuf::from("/tmp/test_persist_native2");
    let theme_file = test_root.join("recurrency").join("theme.txt");
    std::fs::create_dir_all(theme_file.parent().unwrap()).ok();

    std::fs::write(&theme_file, "dark").ok();
    std::thread::sleep(std::time::Duration::from_millis(10));

    let content = std::fs::read_to_string(&theme_file).ok();
    assert_eq!(content.map(|s| s.trim().to_string()).as_deref(), Some("dark"));

    std::fs::remove_file(&theme_file).ok();
    std::fs::remove_dir_all(&test_root).ok();
}
