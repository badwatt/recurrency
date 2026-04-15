let backendUrl = "";
let initialized = false;

const STORAGE_KEY = "recurrency-backend-url";

function isTauri(): boolean {
  return !!(window as any).__TAURI_INTERNALS__;
}

async function loadFromStore(key: string): Promise<string | null> {
  try {
    const { load } = await import("@tauri-apps/plugin-store");
    const store = await load("settings.json");
    const value = await store.get<string>(key);
    return value ?? null;
  } catch {
    return null;
  }
}

async function saveToStore(key: string, value: string): Promise<void> {
  try {
    const { load } = await import("@tauri-apps/plugin-store");
    const store = await load("settings.json");
    await store.set(key, value);
    await store.save();
  } catch {
    // Fallback to localStorage
    localStorage.setItem(STORAGE_KEY, value);
  }
}

export async function initConfig(): Promise<void> {
  if (initialized) return;

  const envUrl = import.meta.env.VITE_API_URL || "";

  if (isTauri()) {
    // Tauri (mobile or desktop) — try store first, then env
    const stored = await loadFromStore(STORAGE_KEY);
    backendUrl = stored || envUrl;
  } else {
    // Web — try localStorage, then env
    const stored = localStorage.getItem(STORAGE_KEY);
    backendUrl = stored || envUrl;
  }

  initialized = true;
}

export function getApiBaseUrl(): string {
  if (!initialized) {
    throw new Error("initConfig() must be called before getApiBaseUrl()");
  }
  return backendUrl;
}

export async function setApiBaseUrl(url: string): Promise<void> {
  backendUrl = url;

  if (isTauri()) {
    await saveToStore(STORAGE_KEY, url);
  } else {
    localStorage.setItem(STORAGE_KEY, url);
  }
}

export function isMobilePlatform(): boolean {
  if (!isTauri()) return false;
  // Synchronous best-effort check; initConfig handles the async platform detection
  return initialized && backendUrl !== "" && !window.matchMedia("(min-width: 1024px)").matches;
}

export function isTauriApp(): boolean {
  return isTauri();
}

export function isBackendUrlConfigured(): boolean {
  return backendUrl !== "";
}