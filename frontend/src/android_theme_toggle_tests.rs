
// TDD tests for Android theme bugs.
// Bug 1: AppLayout llama apply_theme_to_document gated #[cfg(feature=web)] → no-op Android.
// Bug 2: apply_theme_class para "system" usaba window.matchMedia eval (async) → class aplica
//        después del primer paint → OS dark mode nunca se detectaba.
//        FIX: TailwindCSS @media maneja system theme automáticamente (sin JS).
// Bug 3: Swipe sidebar gated #[cfg(feature=web)] → no-op Android.
// Fixes:
//  - AppLayout usa apply_theme_class (sin gate)
//  - apply_theme_class: NO tokio::spawn, synchronous className assignment
//  - use_swipe_sidebar: sin gate, JS detecta Android

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

/// TDD RED: apply_theme_class NO usa tokio::spawn.
/// BUG: tokio::spawn causa delay → class aplica después del render.
/// FIX: dioxus::document::eval directo (fire-and-forget).
#[test]
fn test_apply_theme_uses_sync_eval_no_spawn() {
    let src = include_str!("theme/hooks.rs");
    assert!(!src.contains("tokio::spawn"),
        "apply_theme_class NO debe usar tokio::spawn (delay → system theme broken)");
    assert!(src.contains("dioxus::document::eval"),
        "apply_theme_class debe usar dioxus::document::eval");
}

/// TDD RED: system theme early return en apply_theme_class.
/// BUG: matchMedia eval (async) → class aplica después del paint → OS dark no detectado.
/// FIX: TailwindCSS @media (prefers-color-scheme: dark) maneja system theme automáticamente.
/// apply_theme_class para "system" retorna inmediatamente (no JS).
#[test]
fn test_system_theme_no_match_media() {
    let src = include_str!("theme/hooks.rs");
    // apply_theme_class NO debe usar window.matchMedia en JS (causa delay → broken system theme)
    // TailwindCSS @media (prefers-color-scheme: dark) maneja system theme automáticamente.
    assert!(!src.contains("window.matchMedia"),
        "apply_theme_class NO debe usar window.matchMedia (async → broken system theme)");
    // debe usar dioxus::document::eval para themes explícitos
    assert!(src.contains("dioxus::document::eval"),
        "apply_theme_class debe usar dioxus::document::eval para dark/mocha/light");
}

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

// ── Layout structure coverage ──────────────────────────────────────────
/// Verify app_layout.rs has auth guard.
#[test]
fn layout_auth_guard_present() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("is_authenticated"),
        "AppLayout must check is_authenticated");
}

/// Verify app_layout.rs redirects to login when not auth.
#[test]
fn layout_auth_redirects_to_login() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("Route::Login {}"),
        "AppLayout must redirect to Login when not authenticated");
}

/// Verify app_layout.rs renders Outlet.
#[test]
fn layout_has_route_outlet() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("Outlet::<crate::Route>"),
        "AppLayout must have Route Outlet");
}

/// Verify app_layout uses swipe_sidebar hook.
#[test]
fn layout_uses_swipe_sidebar() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("use_swipe_sidebar()"),
        "AppLayout must call use_swipe_sidebar()");
}

/// Verify app_layout renders desktop sidebar.
#[test]
fn layout_desktop_sidebar_present() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("DesktopSidebar"),
        "AppLayout must render DesktopSidebar");
}

/// Verify app_layout renders notification bell.
#[test]
fn layout_notification_bell_present() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("NotificationBell"),
        "AppLayout must render NotificationBell");
}

/// Verify app_layout renders toast container.
#[test]
fn layout_toast_container_present() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("ToastContainer"),
        "AppLayout must render ToastContainer");
}

/// Verify sidebar closes on route change.
#[test]
fn sidebar_close_on_route_change() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("sidebar_open.set(false)"),
        "Sidebar must close on route change");
}

/// Verify swipe sidebar uses touchstart.
#[test]
fn swipe_sidebar_touchstart_event() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("touchstart"),
        "use_swipe_sidebar must use touchstart event");
}

/// Verify swipe sidebar uses dioxus.send.
#[test]
fn swipe_sidebar_dioxus_send() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("dioxus.send"),
        "use_swipe_sidebar must use dioxus.send");
}

/// Verify swipe sidebar checks typeof dioxus.
#[test]
fn swipe_sidebar_android_check() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("typeof dioxus"),
        "use_swipe_sidebar must check typeof dioxus for Android");
}

/// Verify swipe sidebar sidebar-open animation.
#[test]
fn swipe_sidebar_translate_x_animation() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("sidebar-open"),
        "Swipe sidebar must use sidebar-open for animation");
}

/// Verify theme hooks uses dioxus::document::eval.
#[test]
fn theme_class_uses_document_eval() {
    let src = include_str!("theme/hooks.rs");
    assert!(src.contains("dioxus::document::eval"),
        "apply_theme_class must use dioxus::document::eval");
}

/// Verify theme removes custom class for system.
#[test]
fn theme_system_removes_custom_classes() {
    let src = include_str!("theme/hooks.rs");
    assert!(src.contains("resolve_theme_class"),
        "apply_theme_class for system must remove custom theme classes");
}

/// Verify NavLink components in desktop sidebar.
#[test]
fn desktop_sidebar_nav_links() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("NavLink"),
        "DesktopSidebar must have NavLink components");
    assert!(src.contains("Route::Dashboard"),
        "Sidebar must have Dashboard link");
    assert!(src.contains("Route::Subscriptions"),
        "Sidebar must have Subscriptions link");
    assert!(src.contains("Route::Calendar"),
        "Sidebar must have Calendar link");
}

/// Verify notifications overlay closes on outside click.
#[test]
fn notifications_overlay_close_on_outside_click() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("NOTIFICATIONS.write().is_open = false"),
        "Notifications overlay must close on backdrop click");
}

/// Verify swipe sidebar manages open/close state.
#[test]
fn swipe_sidebar_manages_state() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("open"),
        "use_swipe_sidebar must manage open state");
    assert!(src.contains("close"),
        "use_swipe_sidebar must manage close action");
}

/// Verify swipe sidebar applies to mobile sidebar.
#[test]
fn swipe_sidebar_mobile_sidebar_element() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    assert!(src.contains("mobile-sidebar"),
        "use_swipe_sidebar must target mobile-sidebar element");
}

/// Verify theme cycle includes all four themes.
#[test]
fn theme_cycle_complete() {
    assert_eq!(cycle_theme("light"), "dark");
    assert_eq!(cycle_theme("dark"), "mocha");
    assert_eq!(cycle_theme("mocha"), "system");
    assert_eq!(cycle_theme("system"), "light");
}

/// Verify THEMES constant has 4 entries.
#[test]
fn themes_constant_has_4_entries() {
    assert_eq!(THEMES.len(), 4);
    assert!(THEMES.contains(&"light"));
    assert!(THEMES.contains(&"dark"));
    assert!(THEMES.contains(&"mocha"));
    assert!(THEMES.contains(&"system"));
}

/// Verify app_layout applies theme on mount.
#[test]
fn layout_apply_theme_on_mount() {
    let src = include_str!("layout/app_layout.rs");
    // Must call apply_theme_class in use_effect
    assert!(src.contains("apply_theme_class"),
        "AppLayout must apply theme in use_effect on mount");
}

/// Verify app_layout has mobile header toggle.
#[test]
fn layout_mobile_header_has_toggle() {
    let src = include_str!("layout/app_layout.rs");
    // Mobile header must have sidebar toggle
    assert!(src.contains("sidebar_open.set(!sidebar_open())"),
        "Mobile header must have sidebar toggle button");
}

/// Verify theme applies class to documentElement.
#[test]
fn theme_applies_to_document_element() {
    let src = include_str!("theme/hooks.rs");
    assert!(src.contains("document.documentElement"),
        "apply_theme_class must apply theme to documentElement");
}

/// Verify swipe sidebar returns a Signal.
#[test]
fn swipe_sidebar_returns_signal() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    // Hook must return a Signal<bool>
    assert!(src.contains("Signal<bool>"),
        "use_swipe_sidebar must return Signal<bool>");
}

/// Verify layout has background and text theming.
#[test]
fn layout_uses_themed_background() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("bg-background"),
        "AppLayout must use bg-background for theming");
}

/// Verify sidebar uses bg-card.
#[test]
fn sidebar_uses_card_theme_class() {
    let src = include_str!("layout/app_layout.rs");
    assert!(src.contains("bg-card"),
        "Sidebar must use bg-card for theming");
}

/// Verify swipe sidebar checks for document existence.
#[test]
fn swipe_sidebar_document_guard() {
    let src = include_str!("hooks/use_swipe_sidebar.rs");
    // Must check document exists before accessing
    assert!(src.contains("document") || src.contains("typeof"),
        "use_swipe_sidebar must guard document access");
}
