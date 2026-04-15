import { useEffect, useMemo, useCallback } from "react";
import { CreditCard, Receipt, Calendar, TrendingUp } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { useAuthStore } from "@/stores/authStore";
import { formatCurrency } from "@/lib/format";
import { handleImgError } from "@/lib/utils";

export function DashboardPage() {
  const { user } = useAuthStore();
  const { subscriptions, fetchSubscriptions, isLoading } = useSubscriptionsStore();

  useEffect(() => {
    fetchSubscriptions();
  }, [fetchSubscriptions]);

  const stats = useMemo(() => {
    const active = subscriptions.filter((s) => s.is_active);
    const monthlyTotal = active.reduce((sum, sub) => {
      let m = parseFloat(sub.amount.toString());
      switch (sub.frequency) {
        case "weekly": return sum + m * 4.33;
        case "quarterly": return sum + m / 3;
        case "yearly": return sum + m / 12;
        default: return sum + m;
      }
    }, 0);

    const now = new Date();
    const weekFromNow = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);
    const monthEnd = new Date(now.getFullYear(), now.getMonth() + 1, 0);

    return {
      totalActive: active.length,
      monthlyTotal,
      yearlyTotal: monthlyTotal * 12,
      upcomingThisWeek: active.filter(
        (s) => s.next_billing_date && new Date(s.next_billing_date) >= now && new Date(s.next_billing_date) <= weekFromNow
      ).length,
      upcomingThisMonth: active.filter(
        (s) => s.next_billing_date && new Date(s.next_billing_date) >= now && new Date(s.next_billing_date) <= monthEnd
      ).length,
      subscriptions: active.slice(0, 5),
    };
  }, [subscriptions]);

  const fc = useCallback((amount: number) => formatCurrency(amount, user?.currency || "EUR"), [user?.currency]);

  const getNextBillingDays = (date: string | null) => {
    if (!date) return null;
    const diff = Math.ceil((new Date(date).getTime() - Date.now()) / 86400000);
    if (diff === 0) return "Today";
    if (diff === 1) return "Tomorrow";
    if (diff < 0) return "Overdue";
    return `${diff} days`;
  };

  const renderSubEntry = (sub: typeof subscriptions[0]) => (
    <div key={sub.id} className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-full bg-primary/10">
          <CreditCard className="h-4 w-4 text-primary" />
        </div>
        <div>
          <p className="text-sm font-medium">{sub.name}</p>
          <p className="text-xs text-muted-foreground capitalize">{sub.frequency}</p>
        </div>
      </div>
      <div className="text-right">
        <p className="text-sm font-medium">{fc(parseFloat(sub.amount.toString()))}</p>
        <p className="text-xs text-muted-foreground">{getNextBillingDays(sub.next_billing_date)}</p>
      </div>
    </div>
  );

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Welcome back, {user?.name || user?.email}</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Monthly Subscriptions</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{fc(stats.monthlyTotal)}</div>
            <p className="text-xs text-muted-foreground">{stats.totalActive} active subscriptions</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Yearly Cost</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{fc(stats.yearlyTotal)}</div>
            <p className="text-xs text-muted-foreground">Projected annual spending</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Due This Week</CardTitle>
            <Calendar className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.upcomingThisWeek}</div>
            <p className="text-xs text-muted-foreground">Payments in next 7 days</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Due This Month</CardTitle>
            <Receipt className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.upcomingThisMonth}</div>
            <p className="text-xs text-muted-foreground">Payments remaining this month</p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Active Subscriptions</CardTitle></CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex h-32 items-center justify-center"><div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" /></div>
            ) : stats.subscriptions.length === 0 ? (
              <p className="flex h-32 items-center justify-center text-muted-foreground">No subscriptions yet</p>
            ) : (
              <div className="space-y-4">{stats.subscriptions.map(renderSubEntry)}</div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Upcoming Payments</CardTitle></CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex h-32 items-center justify-center"><div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" /></div>
            ) : subscriptions.filter((s) => s.is_active).length === 0 ? (
              <p className="flex h-32 items-center justify-center text-muted-foreground">No upcoming payments</p>
            ) : (
              <div className="space-y-4">
                {subscriptions
                  .filter((s) => s.is_active && s.next_billing_date)
                  .sort((a, b) => new Date(a.next_billing_date!).getTime() - new Date(b.next_billing_date!).getTime())
                  .slice(0, 5)
                  .map((sub) => {
                    const nextDate = new Date(sub.next_billing_date!);
                    const today = new Date();
                    const isOverdue = nextDate < today;
                    const isToday = nextDate.toDateString() === today.toDateString();
                    return (
                      <div key={sub.id} className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className={`flex h-9 w-9 items-center justify-center rounded-full overflow-hidden ${
                            isOverdue ? "bg-destructive/10" : isToday ? "bg-yellow-500/10" : "bg-green-500/10"
                          }`}>
                            {sub.logo_url ? (
                              <img
                                src={sub.logo_url}
                                alt={sub.name}
                                className="h-full w-full object-cover"
                                onError={handleImgError}
                              />
                            ) : (
                              <Receipt className={`h-4 w-4 ${isToday ? "text-yellow-500" : isOverdue ? "text-destructive" : "text-green-500"}`} />
                            )}
                          </div>
                          <div>
                            <p className="text-sm font-medium">{sub.name}</p>
                            <p className="text-xs text-muted-foreground">
                              {nextDate.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" })}
                            </p>
                          </div>
                        </div>
                        <div className="text-right">
                          <p className="text-sm font-medium">{fc(parseFloat(sub.amount.toString()))}</p>
                          <p className={`text-xs ${isOverdue ? "text-destructive" : "text-muted-foreground"}`}>
                            {getNextBillingDays(sub.next_billing_date)}
                          </p>
                        </div>
                      </div>
                    );
                  })}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
