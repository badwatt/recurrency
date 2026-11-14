# Recurrency tao patch

This is tao 0.34.8, used by dioxus-desktop 0.7.10.

The only change is in `src/platform_impl/android/mod.rs`: `MonitorHandle::size`
now queries `getCurrentWindowMetrics` only on Android API 30+ and falls back to
the legacy `getDefaultDisplay` metrics on older releases.

Without this guard, tao calls a method that does not exist on Android 10 and
panics at startup. Huawei P30 Pro (EMUI 10 / Android 10) is one affected device.
Upstream fix: tauri-apps/tao#1211 (released in tao 0.37.1).
