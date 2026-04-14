import { Suspense, useState, useEffect, lazy } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { Toaster } from "sonner";
import { useAuth } from "@/hooks/useAuth";
import { initConfig, isMobilePlatform, isBackendUrlConfigured } from "@/lib/config";
import { LoadingScreen } from "@/components/loading-screen";

// Layout
import { AppLayout } from "@/pages/layout/AppLayout";

// Pages (eager: auth + setup — needed immediately on first load)
import { LoginPage } from "@/pages/auth/LoginPage";
import { RegisterPage } from "@/pages/auth/RegisterPage";
import { SetupPage } from "@/pages/SetupPage";

// Pages (lazy: protected routes — loaded on demand with Suspense fallback)
const DashboardPage = lazy(() => import("@/pages/DashboardPage").then(m => ({ default: m.DashboardPage })));
const SubscriptionsPage = lazy(() => import("@/pages/SubscriptionsPage").then(m => ({ default: m.SubscriptionsPage })));
const TransactionsPage = lazy(() => import("@/pages/TransactionsPage").then(m => ({ default: m.TransactionsPage })));
const CalendarPage = lazy(() => import("@/pages/CalendarPage").then(m => ({ default: m.CalendarPage })));
const CategoriesPage = lazy(() => import("@/pages/CategoriesPage").then(m => ({ default: m.CategoriesPage })));
const ReportsPage = lazy(() => import("@/pages/ReportsPage").then(m => ({ default: m.ReportsPage })));
const SettingsPage = lazy(() => import("@/pages/SettingsPage").then(m => ({ default: m.SettingsPage })));

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return <LoadingScreen />;
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
    return <LoadingScreen />;
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
      <Suspense fallback={<LoadingScreen />}>
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
      </Suspense>
      <Toaster position="top-right" richColors />
    </>
  );
}

export default App;