import { useEffect, useState, useMemo } from "react";
import { ChevronLeft, ChevronRight, CreditCard, Calendar as CalendarIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useAuthStore } from "@/stores/authStore";
import { calendarApi, type CalendarItem } from "@/lib/api/calendar";
import { SubscriptionDialog } from "@/components/subscription-dialog";
import { subscriptionsApi } from "@/lib/api/subscriptions";

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December"
];

export function CalendarPage() {
  const { user } = useAuthStore();
  const [currentDate, setCurrentDate] = useState(new Date());
  const [calendarItems, setCalendarItems] = useState<CalendarItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);

  const year = currentDate.getFullYear();
  const month = currentDate.getMonth() + 1;

  useEffect(() => {
    fetchCalendar();
  }, [year, month]);

  const fetchCalendar = async () => {
    setIsLoading(true);
    try {
      const items = await calendarApi.getMonth(year, month);
      setCalendarItems(items);
    } catch (error) {
      console.error("Failed to fetch calendar:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const goToPrevMonth = () => setCurrentDate(new Date(year, month - 2, 1));
  const goToNextMonth = () => setCurrentDate(new Date(year, month, 1));
  const goToToday = () => setCurrentDate(new Date());

  const calendarDays = useMemo(() => {
    const firstDay = new Date(year, month - 1, 1);
    const lastDay = new Date(year, month, 0);
    const startPadding = firstDay.getDay();
    const totalDays = lastDay.getDate();
    const days: (number | null)[] = [];
    for (let i = 0; i < startPadding; i++) days.push(null);
    for (let i = 1; i <= totalDays; i++) days.push(i);
    return days;
  }, [year, month]);

  const getItemsForDay = (day: number | null): CalendarItem[] => {
    if (!day) return [];
    const dateStr = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    return calendarItems.filter(item => item.date === dateStr);
  };

  const formatCurrency = (amount: string, currency: string) =>
    new Intl.NumberFormat("en-US", { style: "currency", currency }).format(parseFloat(amount));

  const today = new Date();
  const isToday = (day: number) =>
    day === today.getDate() && month === today.getMonth() + 1 && year === today.getFullYear();

  const handleDayClick = (day: number | null) => {
    if (day === null) return;
    const dateStr = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    setSelectedDate(dateStr);
    setDialogOpen(true);
  };

  const handleDialogClose = (open: boolean) => {
    setDialogOpen(open);
    if (!open) setSelectedDate(null);
    else fetchCalendar();
  };

  const handleSubmitSubscription = async (data: any) => {
    await subscriptionsApi.create(data);
  };

  // Sort upcoming items by date
  const upcomingItems = useMemo(() => 
    [...calendarItems].sort((a, b) => a.date.localeCompare(b.date)),
    [calendarItems]
  );

  const monthTotal = calendarItems.reduce((sum, item) => sum + parseFloat(item.amount.toString()), 0);

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Calendar</h1>
          <p className="text-muted-foreground">
            View your upcoming subscription payments
          </p>
        </div>
        <div className="text-right hidden md:block">
          <p className="text-2xl font-bold">{formatCurrency(monthTotal.toString(), user?.currency || "USD")}</p>
          <p className="text-sm text-muted-foreground">Total this month</p>
        </div>
      </div>

      {/* Mobile: Upcoming payments list */}
      <div className="md:hidden space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <CalendarIcon className="h-5 w-5" />
            Upcoming Payments
          </h2>
          <p className="text-lg font-bold">{formatCurrency(monthTotal.toString(), user?.currency || "USD")}</p>
        </div>
        
        {isLoading ? (
          <div className="flex h-48 items-center justify-center">
            <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          </div>
        ) : upcomingItems.length === 0 ? (
          <Card><CardContent className="py-8 text-center text-muted-foreground">No payments scheduled this month</CardContent></Card>
        ) : (
          <div className="space-y-2">
            {upcomingItems.map((item, index) => (
              <Card key={index} className="cursor-pointer hover:bg-muted/50" onClick={() => { setSelectedDate(item.date); setDialogOpen(true); }}>
                <CardContent className="p-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
                        <CreditCard className="h-5 w-5 text-primary" />
                      </div>
                      <div>
                        <p className="font-medium">{item.name}</p>
                        <p className="text-sm text-muted-foreground">
                          {new Date(item.date).toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" })}
                        </p>
                      </div>
                    </div>
                    <p className="text-lg font-semibold">{formatCurrency(item.amount, item.currency)}</p>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>

      {/* Desktop: Full calendar */}
      <Card className="hidden md:block">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Button variant="outline" size="icon" onClick={goToPrevMonth}><ChevronLeft className="h-4 w-4" /></Button>
              <h2 className="text-xl font-semibold min-w-[200px] text-center">{MONTHS[month - 1]} {year}</h2>
              <Button variant="outline" size="icon" onClick={goToNextMonth}><ChevronRight className="h-4 w-4" /></Button>
            </div>
            <Button variant="outline" onClick={goToToday}>Today</Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex h-96 items-center justify-center">
              <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
            </div>
          ) : (
            <div className="grid grid-cols-7 gap-px bg-border">
              {WEEKDAYS.map((day) => (
                <div key={day} className="bg-muted p-2 text-center text-sm font-medium text-muted-foreground">{day}</div>
              ))}
              {calendarDays.map((day, index) => {
                const dayItems = getItemsForDay(day);
                return (
                  <div
                    key={index}
                    className={`min-h-[100px] bg-card p-2 hover:bg-muted/50 cursor-pointer transition-colors ${day === null ? "bg-muted/30 cursor-default" : ""}`}
                    onClick={() => day !== null && handleDayClick(day)}
                  >
                    {day !== null && (
                      <>
                        <div className={`mb-1 flex h-7 w-7 items-center justify-center rounded-full text-sm ${isToday(day) ? "bg-primary text-primary-foreground font-bold" : ""}`}>
                          {day}
                        </div>
                        <div className="space-y-1">
                          {dayItems.slice(0, 3).map((item, itemIndex) => (
                            <div key={itemIndex} className="flex items-center gap-1 rounded bg-primary/10 px-1 py-0.5 text-xs" title={`${item.name} - ${formatCurrency(item.amount, item.currency)}`}>
                              {(item as {logo_url?: string}).logo_url ? (
                                <img
                                  src={(item as {logo_url?: string}).logo_url}
                                  alt={item.name}
                                  className="h-3 w-3 flex-shrink-0 object-contain rounded"
                                  onError={(e) => {
                                    (e.target as HTMLImageElement).style.display = "none";
                                  }}
                                />
                              ) : (
                                <CreditCard className="h-3 w-3 flex-shrink-0" />
                              )}
                              <span className="truncate">{item.name}</span>
                              <span className="ml-auto flex-shrink-0 font-medium">${parseFloat(item.amount).toFixed(0)}</span>
                            </div>
                          ))}
                          {dayItems.length > 3 && <div className="text-xs text-muted-foreground">+{dayItems.length - 3} more</div>}
                        </div>
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          )}
          <div className="mt-4 flex items-center gap-4 text-sm text-muted-foreground">
            <div className="flex items-center gap-2"><div className="h-4 w-4 rounded-full bg-primary" /><span>Today</span></div>
            <div className="flex items-center gap-2"><div className="h-4 w-4 rounded bg-primary/10" /><span>Payment due</span></div>
          </div>
        </CardContent>
      </Card>

      <SubscriptionDialog
        open={dialogOpen}
        onOpenChange={handleDialogClose}
        defaultDate={selectedDate || undefined}
        onSubmit={handleSubmitSubscription}
      />
    </div>
  );
}
