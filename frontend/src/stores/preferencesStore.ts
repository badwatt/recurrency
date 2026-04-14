import { create } from "zustand";
import { preferencesApi, type Preferences } from "@/lib/api/preferences";

interface PreferencesState {
  preferences: Preferences;
  isLoading: boolean;
  initialized: boolean;
  
  // Actions
  fetchPreferences: () => Promise<void>;
  updatePreferences: (updates: Partial<Preferences>) => Promise<void>;
  setTheme: (theme: Preferences["theme"]) => Promise<void>;
  setSubscriptionsView: (view: Preferences["subscriptions_view"]) => Promise<void>;
}

export const usePreferencesStore = create<PreferencesState>((set, get) => ({
  preferences: {
    theme: "system",
    subscriptions_view: "grid",
  },
  isLoading: false,
  initialized: false,

  fetchPreferences: async () => {
    if (get().initialized) return;
    
    set({ isLoading: true });
    try {
      const preferences = await preferencesApi.get();
      localStorage.setItem('recurrency-theme', preferences.theme);
      set({ preferences, isLoading: false, initialized: true });
    } catch (error) {
      console.error("Failed to fetch preferences:", error);
      set({ isLoading: false, initialized: true });
    }
  },

  updatePreferences: async (updates: Partial<Preferences>) => {
    try {
      const updated = await preferencesApi.update(updates);
      set({ preferences: updated });
    } catch (error) {
      console.error("Failed to update preferences:", error);
      throw error;
    }
  },

  setTheme: async (theme: Preferences["theme"]) => {
    localStorage.setItem('recurrency-theme', theme);
    set((state) => ({
      preferences: { ...state.preferences, theme },
    }));
    await get().updatePreferences({ theme });
  },

  setSubscriptionsView: async (view: Preferences["subscriptions_view"]) => {
    set((state) => ({
      preferences: { ...state.preferences, subscriptions_view: view },
    }));
    await get().updatePreferences({ subscriptions_view: view });
  },
}));
