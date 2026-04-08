import { useEffect } from "react";
import { useAuthStore } from "@/stores/authStore";

export function useAuth() {
  const {
    user,
    isAuthenticated,
    isLoading,
    initialized,
    initialize,
    login,
    register,
    logout,
  } = useAuthStore();

  useEffect(() => {
    if (!initialized) {
      initialize();
    }
  }, [initialized, initialize]);

  return {
    user,
    isAuthenticated,
    isLoading: isLoading || !initialized,
    initialized,
    login,
    register,
    logout,
  };
}
