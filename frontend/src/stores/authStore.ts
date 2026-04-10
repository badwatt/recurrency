import { create } from "zustand";
import { persist } from "zustand/middleware";
import { auth } from "@/lib/auth";
import type { User } from "@/lib/api/auth";

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  initialized: boolean;
  
  // Actions
  initialize: () => Promise<void>;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, name: string) => Promise<void>;
  logout: () => Promise<void>;
  setLoading: (loading: boolean) => void;
  setUser: (user: User) => void;
  updateUser: (updates: Partial<User>) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      initialized: false,

      initialize: async () => {
        if (get().initialized) return;
        
        set({ isLoading: true });
        
        try {
          const user = await auth.getCurrentUser();
          const isAuth = user !== null;
          
          set({
            user,
            isAuthenticated: isAuth,
            isLoading: false,
            initialized: true,
          });
        } catch {
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            initialized: true,
          });
        }
      },

      login: async (email: string, password: string) => {
        set({ isLoading: true });
        
        try {
          const response = await auth.signIn({ email, password });
          set({
            user: response.user,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error) {
          set({ isLoading: false });
          throw error;
        }
      },

      register: async (email: string, password: string, name: string) => {
        set({ isLoading: true });
        
        try {
          const response = await auth.signUp({ email, password, name });
          set({
            user: response.user,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error) {
          set({ isLoading: false });
          throw error;
        }
      },

      logout: async () => {
        await auth.signOut();
        set({
          user: null,
          isAuthenticated: false,
          isLoading: false,
        });
      },

      updateUser: (updates: Partial<User>) => {
        const currentUser = get().user;
        if (currentUser) {
          const updatedUser = { ...currentUser, ...updates };
          set({ user: updatedUser });
        }
      },

      setLoading: (isLoading: boolean) => set({ isLoading }),

      setUser: (user: User) => set({ user }),
    }),
    {
      name: "recurrency-auth",
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);
