import { useState, useEffect } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { Toaster } from "sonner";
import { useAuth } from "@/hooks/useAuth";
import { initConfig, isMobilePlatform, isBackendUrlConfigured } from "@/lib/config";

// Layout
import { AppLayout } from "@/pages/layout/AppLayout";

// Pages
import { LoginPage } from "@/pages/auth/LoginPage";
import { RegisterPage } from "@/pages/auth/RegisterPage";
import { DashboardPage } from "@/pages/DashboardPage";
import { SubscriptionsPage } from "@/pages/SubscriptionsPage";
import { TransactionsPage } from "@/pages/TransactionsPage";
import { CalendarPage } from "@/pages/CalendarPage";
import { CategoriesPage } from "@/pages/CategoriesPage";
import { ReportsPage } from "@/pages/ReportsPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { SetupPage } from "@/pages/SetupPage";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

function App() {
  const [configReady, setConfigReady] = useState(false);

  useEffect(() => {
    initConfig().then(() => setConfigReady(true));
  }, []);

  if (!configReady) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  // On mobile, if no backend URL is configured, show setup
  if (isMobilePlatform() && !isBackendUrlConfigured()) {
    return (
      <>
        <Routes>
          <Route path="/setup" element={<SetupPage />} />
          <Route path="*" element={<Navigate to="/setup" replace />} />
        </Routes>
        <Toaster position="top-right" richColors />
      </>
    );
  }

  return (
    <>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/setup" element={<SetupPage />} />

        {/* Protected routes */}
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <AppLayout />
            </ProtectedRoute>
          }
        >
          <Route index element={<DashboardPage />} />
          <Route path="subscriptions" element={<SubscriptionsPage />} />
          <Route path="transactions" element={<TransactionsPage />} />
          <Route path="calendar" element={<CalendarPage />} />
          <Route path="categories" element={<CategoriesPage />} />
          <Route path="reports" element={<ReportsPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>

        {/* Catch all */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toaster position="top-right" richColors />
    </>
  );
}

export default App;