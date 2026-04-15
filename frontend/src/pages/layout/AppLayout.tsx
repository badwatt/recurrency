import { useState } from "react";
import { Outlet, NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  CreditCard,
  Receipt,
  Calendar,
  Tag,
  BarChart3,
  Settings,
  LogOut,
  Menu,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/stores/authStore";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { NotificationBell } from "@/components/notification-bell";
import { ThemeToggle } from "@/components/theme-toggle";
import { useSwipeSidebar } from "@/hooks/useSwipeSidebar";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/subscriptions", icon: CreditCard, label: "Subscriptions" },
  { to: "/transactions", icon: Receipt, label: "Transactions" },
  { to: "/calendar", icon: Calendar, label: "Calendar" },
  { to: "/categories", icon: Tag, label: "Categories" },
  { to: "/reports", icon: BarChart3, label: "Reports" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export function AppLayout() {
  const { user, logout } = useAuthStore();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const closeSidebar = () => setSidebarOpen(false);
  const openSidebar = () => setSidebarOpen(true);

  useSwipeSidebar({
    onOpen: openSidebar,
    onClose: closeSidebar,
    isOpen: sidebarOpen,
  });

  const handleLogout = async () => {
    await logout();
  };

  return (
    <div className="flex h-dvh">
      {/* Mobile sidebar backdrop */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm lg:hidden transition-opacity"
          onClick={closeSidebar}
        />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          "fixed lg:static inset-y-0 left-0 z-50 flex w-64 flex-col border-r bg-card",
          "transform transition-transform duration-200 ease-in-out lg:translate-x-0",
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        )}
      >
        {/* Logo */}
        <div className="flex min-h-14 items-center justify-between border-b px-4">
          <h1 className="text-xl font-bold tracking-tight">Recurrency</h1>
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden"
            onClick={closeSidebar}
          >
            <X className="h-5 w-5" />
          </Button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 space-y-1 p-4 overflow-y-auto">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              onClick={closeSidebar}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )
              }
            >
              <item.icon className="h-4 w-4 flex-shrink-0" />
              <span className="truncate">{item.label}</span>
            </NavLink>
          ))}
        </nav>

        <Separator />

        {/* User section */}
        <div className="p-4">
          <div className="flex items-center gap-3">
            <div className="flex flex-1 flex-col min-w-0">
              <span className="text-sm font-medium truncate">{user?.name || user?.email}</span>
              <span className="text-xs text-muted-foreground">{__APP_VERSION__}</span>
            </div>
            <ThemeToggle />
            <Button
              variant="ghost"
              size="icon"
              onClick={handleLogout}
              title="Logout"
            >
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </aside>

      {/* Main content area */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Mobile header - translucent with safe area */}
        <header className="flex min-h-14 items-center justify-between bg-card/80 backdrop-blur-md border-b border-border/50 px-4 lg:hidden sticky top-0 z-30">
          <Button
            variant="ghost"
            size="icon"
            onClick={openSidebar}
          >
            <Menu className="h-5 w-5" />
          </Button>
          <h1 className="text-lg font-bold">Recurrency</h1>
          <div className="flex items-center gap-1">
            <NotificationBell />
          </div>
        </header>

        {/* Desktop header */}
        <header className="hidden lg:flex h-14 items-center justify-end gap-2 bg-card/80 backdrop-blur-md border-b border-border/50 px-6">
          <NotificationBell />
        </header>

        {/* Main content */}
        <main className="flex-1 overflow-auto overflow-x-hidden bg-background">
          <Outlet />
        </main>
      </div>
    </div>
  );
}