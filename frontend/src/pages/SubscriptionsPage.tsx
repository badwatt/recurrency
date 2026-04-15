import { useEffect, useCallback, useState, useMemo } from "react";
import { Plus, Pencil, Trash2, LayoutGrid, List, CreditCard, StickyNote, Pause, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useCategoriesStore } from "@/stores/categoriesStore";
import { useAuthStore } from "@/stores/authStore";
import { SubscriptionDialog } from "@/components/subscription-dialog";
import { NotesEditDialog } from "@/components/notes-edit-dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { FilterSort, type FilterConfig, type SortConfig } from "@/components/filter-sort";
import type { Subscription, CreateSubscriptionRequest } from "@/lib/api/subscriptions";
import { showToast } from "@/lib/toast";
import { formatCurrency } from "@/lib/format";
import { getIcon } from "@/lib/icons";

type ViewMode = "grid" | "list";



interface SubscriptionCardProps {
  subscription: Subscription;
  viewMode: ViewMode;
  onEdit: (subscription: Subscription) => void;
  onDelete: (subscription: Subscription) => void;
  onToggleActive: (subscription: Subscription) => void;
  onNotesClick: (subscription: Subscription) => void;
  formatCurrency: (amount: number, currency: string) => string;
  getCategoryStyle: (categoryId: string) => { color: string; icon: string; name: string };
}

function SubscriptionCard({
  subscription,
  viewMode,
  onEdit,
  onDelete,
  onToggleActive,
  onNotesClick,
  formatCurrency,
  getCategoryStyle,
}: SubscriptionCardProps) {
  const { color, icon, name: catName } = getCategoryStyle(subscription.category_id);
  const IconComponent = getIcon(icon);

  if (viewMode === "grid") {
    return (
      <Card className={`group relative flex flex-col h-full ${!subscription.is_active ? "opacity-60" : ""}`}>
        <CardContent className="flex flex-col items-center justify-between flex-1 p-5 pb-4 text-center gap-3">
          {/* Logo / Icon */}
          <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-muted overflow-hidden">
            {subscription.logo_url ? (
              <img
                src={subscription.logo_url}
                alt={subscription.name}
                className="h-full w-full object-cover"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                }}
              />
            ) : (
              <CreditCard className="h-7 w-7 text-muted-foreground" />
            )}
          </div>

          {/* Name & billing date */}
          <div className="min-w-0 w-full">
            <p className="font-semibold truncate text-base leading-tight">{subscription.name}</p>
            <p className="text-xs text-muted-foreground mt-0.5 truncate">
              {subscription.is_active && subscription.next_billing_date
                ? `Next: ${new Date(subscription.next_billing_date).toLocaleDateString()}`
                : subscription.is_active ? "No billing date" : "Inactive"}
            </p>
          </div>

          {/* Amount + Notes (mobile) */}
          <div className="flex items-center gap-1">
            {subscription.notes && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-6 w-6 sm:hidden" onClick={() => onNotesClick(subscription)}>
                      <StickyNote className="h-3 w-3 text-muted-foreground" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" className="max-w-[280px] whitespace-pre-wrap text-xs">
                    {subscription.notes}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
            <p className="font-bold text-xl tracking-tight">
              {formatCurrency(subscription.amount, subscription.currency)}
            </p>
          </div>

          {/* Category badge */}
          <Badge
            variant="secondary"
            className="font-normal text-xs"
            style={{
              color: color || "#6b7280",
              backgroundColor: `${color}15` || "#f3f4f6",
            }}
          >
            <IconComponent className="h-3 w-3 mr-1" />
            {catName}
          </Badge>

          {/* Actions */}
          <div className="flex items-center gap-1 mt-auto flex-wrap justify-center">
            {subscription.notes && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="hidden sm:flex h-8 w-8" onClick={() => onNotesClick(subscription)}>
                      <StickyNote className="h-4 w-4 text-muted-foreground" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" className="max-w-[280px] whitespace-pre-wrap text-xs">
                    {subscription.notes}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
            <TooltipProvider delayDuration={200}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onToggleActive(subscription)}>
                    {subscription.is_active
                      ? <Pause className="h-4 w-4 text-amber-500" />
                      : <Play className="h-4 w-4 text-emerald-500" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                  {subscription.is_active ? "Pause subscription" : "Resume subscription"}
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onEdit(subscription)}>
              <Pencil className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onDelete(subscription)}>
              <Trash2 className="h-4 w-4 text-destructive" />
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  // List view
  return (
    <Card className={!subscription.is_active ? "opacity-60" : "min-w-0"}>
      <CardContent className="p-3 sm:p-4">
        <div className="flex items-center gap-2 sm:gap-4">
          <div className="flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden">
            {subscription.logo_url ? (
              <img
                src={subscription.logo_url}
                alt={subscription.name}
                className="h-full w-full object-cover"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                }}
              />
            ) : (
              <CreditCard className="h-4 w-4 sm:h-5 sm:w-5 text-muted-foreground" />
            )}
          </div>
          <div className="flex-1 min-w-0">
            <p className="font-medium truncate text-sm sm:text-base">{subscription.name}</p>
            <p className="text-xs text-muted-foreground truncate">
              {subscription.is_active && subscription.next_billing_date
                ? `Next: ${new Date(subscription.next_billing_date).toLocaleDateString()}`
                : subscription.is_active ? "No billing date" : "Inactive"}
            </p>
          </div>
          <div className="flex items-center gap-2 sm:gap-4 shrink-0">
            {subscription.notes && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => onNotesClick(subscription)}>
                      <StickyNote className="h-4 w-4 text-muted-foreground" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" className="max-w-[280px] whitespace-pre-wrap text-xs">
                    {subscription.notes}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
            <Badge
              variant="secondary"
              className="font-normal text-xs hidden sm:inline-flex"
              style={{
                color: color || "#6b7280",
                backgroundColor: `${color}15` || "#f3f4f6",
              }}
            >
              <IconComponent className="h-3 w-3 mr-1" />
              {catName}
            </Badge>
            <p className="font-medium text-sm sm:text-base">
              {formatCurrency(subscription.amount, subscription.currency)}
            </p>
            <div className="flex items-center gap-0.5 sm:gap-1 shrink-0">
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-7 w-7 sm:h-8 sm:w-8" onClick={() => onToggleActive(subscription)}>
                      {subscription.is_active
                        ? <Pause className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-amber-500" />
                        : <Play className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-emerald-500" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom">
                    {subscription.is_active ? "Pause subscription" : "Resume subscription"}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
              <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => onEdit(subscription)}>
                <Pencil className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => onDelete(subscription)}>
                <Trash2 className="h-4 w-4 text-destructive" />
              </Button>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function SubscriptionsPage() {
  const { user } = useAuthStore();
  const {
    subscriptions,
    isLoading,
    error,
    fetchSubscriptions,
    createSubscription,
    updateSubscription,
    deleteSubscription,
  } = useSubscriptionsStore();

  const { preferences, fetchPreferences, setSubscriptionsView } = usePreferencesStore();
  const { categories, fetchCategories } = useCategoriesStore();
  const viewMode = preferences.subscriptions_view as ViewMode;

  const getCategoryStyle = useCallback(
    (categoryId: string) => {
      const cat = categories.find((c) => c.id === categoryId);
      if (!cat) return { color: "#6b7280", icon: "", name: "Uncategorized" };
      return {
        color: cat.color || "#6b7280",
        icon: cat.icon || "",
        name: cat.name,
      };
    },
    [categories]
  );

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingSubscription, setEditingSubscription] = useState<Subscription | undefined>();
  const [notesEditOpen, setNotesEditOpen] = useState(false);
  const [notesEditingSubscription, setNotesEditingSubscription] = useState<Subscription | undefined>();

  const [filters, setFilters] = useState<FilterConfig>({});
  const [sort, setSort] = useState<SortConfig>({ field: "next_billing_date", direction: "asc" });

  useEffect(() => {
    fetchSubscriptions();
    fetchPreferences();
    fetchCategories();
  }, [fetchSubscriptions, fetchPreferences, fetchCategories]);

  const handleCreate = () => {
    setEditingSubscription(undefined);
    setDialogOpen(true);
  };

  const handleEdit = (subscription: Subscription) => {
    setEditingSubscription(subscription);
    setDialogOpen(true);
  };

  const handleDelete = useCallback(
    async (subscription: Subscription) => {
      if (!confirm(`Are you sure you want to delete "${subscription.name}"?`)) {
        return;
      }

      try {
        await deleteSubscription(subscription.id);
        showToast("success", `"${subscription.name}" deleted`);
      } catch {
        showToast("error", "Failed to delete subscription");
      }
    },
    [deleteSubscription]
  );

  const handleToggleActive = useCallback(
    async (subscription: Subscription) => {
      try {
        await updateSubscription(subscription.id, { is_active: !subscription.is_active });
        showToast(
          "success",
          subscription.is_active
            ? `"${subscription.name}" paused`
            : `"${subscription.name}" resumed`
        );
      } catch {
        showToast("error", "Failed to update subscription");
      }
    },
    [updateSubscription]
  );

  const handleNotesClick = useCallback(
    (subscription: Subscription) => {
      setNotesEditingSubscription(subscription);
      setNotesEditOpen(true);
    },
    []
  );

  const handleNotesSave = useCallback(
    async (notes: string) => {
      if (!notesEditingSubscription) return;
      await updateSubscription(notesEditingSubscription.id, { notes });
      showToast("success", "Notes updated");
    },
    [notesEditingSubscription, updateSubscription]
  );

  const handleSubmit = useCallback(
    async (data: CreateSubscriptionRequest) => {
      try {
        if (editingSubscription) {
          await updateSubscription(editingSubscription.id, data);
          showToast("success", "Subscription updated");
        } else {
          await createSubscription(data);
          showToast("success", "Subscription added");
        }
      } catch {
        showToast("error", editingSubscription ? "Failed to update" : "Failed to create subscription");
      }
    },
    [editingSubscription, createSubscription, updateSubscription]
  );

  const statusOptions = [
    { value: "all", label: "All" },
    { value: "active", label: "Active" },
    { value: "inactive", label: "Inactive" },
  ];

  const frequencyOptions = [
    { value: "weekly", label: "Weekly" },
    { value: "monthly", label: "Monthly" },
    { value: "quarterly", label: "Quarterly" },
    { value: "yearly", label: "Yearly" },
  ];

  const sortOptions = [
    { value: "next_billing_date", label: "Upcoming date" },
    { value: "amount", label: "Price" },
    { value: "name", label: "Name" },
    { value: "frequency", label: "Frequency" },
    { value: "start_date", label: "Start date" },
  ];

  const filteredSubscriptions = useMemo(() => {
    let result = [...subscriptions];

    if (filters.status === "active") result = result.filter((s) => s.is_active);
    else if (filters.status === "inactive") result = result.filter((s) => !s.is_active);

    if (filters.frequency) result = result.filter((s) => s.frequency === filters.frequency);
    if (filters.categoryId) result = result.filter((s) => s.category_id === filters.categoryId);

    if (filters.priceMin) {
      const min = parseFloat(filters.priceMin);
      if (!isNaN(min)) result = result.filter((s) => s.amount >= min);
    }
    if (filters.priceMax) {
      const max = parseFloat(filters.priceMax);
      if (!isNaN(max)) result = result.filter((s) => s.amount <= max);
    }

    const freqOrder: Record<string, number> = { weekly: 0, monthly: 1, quarterly: 2, yearly: 3 };

    result.sort((a, b) => {
      let cmp = 0;
      switch (sort.field) {
        case "next_billing_date": {
          const aDate = a.next_billing_date ? new Date(a.next_billing_date).getTime() : Infinity;
          const bDate = b.next_billing_date ? new Date(b.next_billing_date).getTime() : Infinity;
          cmp = aDate - bDate;
          break;
        }
        case "amount":
          cmp = a.amount - b.amount;
          break;
        case "name":
          cmp = a.name.localeCompare(b.name);
          break;
        case "frequency":
          cmp = (freqOrder[a.frequency] ?? 99) - (freqOrder[b.frequency] ?? 99);
          break;
        case "start_date": {
          const aStart = a.start_date ? new Date(a.start_date).getTime() : Infinity;
          const bStart = b.start_date ? new Date(b.start_date).getTime() : Infinity;
          cmp = aStart - bStart;
          break;
        }
      }
      return sort.direction === "asc" ? cmp : -cmp;
    });

    return result;
  }, [subscriptions, filters, sort]);

  const clearFilters = () => setFilters({});

  const renderSubscriptions = () => {
    if (viewMode === "grid") {
      return (
        <div className="grid gap-4 grid-cols-2 lg:grid-cols-4">
          {filteredSubscriptions.map((subscription) => (
            <SubscriptionCard
              key={subscription.id}
              subscription={subscription}
              viewMode={viewMode}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onToggleActive={handleToggleActive}
              onNotesClick={handleNotesClick}
              formatCurrency={formatCurrency}
              getCategoryStyle={getCategoryStyle}
            />
          ))}
        </div>
      );
    }

    return (
      <div className="space-y-2">
        {filteredSubscriptions.map((subscription) => (
          <SubscriptionCard
            key={subscription.id}
            subscription={subscription}
            viewMode={viewMode}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleActive={handleToggleActive}
            onNotesClick={handleNotesClick}
            formatCurrency={formatCurrency}
            getCategoryStyle={getCategoryStyle}
          />
        ))}
      </div>
    );
  };

  if (error) {
    return (
      <div className="p-4 md:p-6">
        <div className="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start justify-between gap-3 sm:flex-col sm:gap-0">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Subscriptions</h1>
            <p className="text-muted-foreground">
              Manage your recurring expenses
            </p>
          </div>
          <div className="flex border rounded-md shrink-0 sm:hidden">
            <Button
              variant={viewMode === "grid" ? "secondary" : "ghost"}
              size="icon"
              onClick={() => setSubscriptionsView("grid")}
              title="Grid view"
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "list" ? "secondary" : "ghost"}
              size="icon"
              onClick={() => setSubscriptionsView("list")}
              title="List view"
            >
              <List className="h-4 w-4" />
            </Button>
          </div>
        </div>
        <div className="flex items-center gap-2 sm:w-auto w-full">
          <div className="hidden sm:flex border rounded-md shrink-0">
            <Button
              variant={viewMode === "grid" ? "secondary" : "ghost"}
              size="icon"
              onClick={() => setSubscriptionsView("grid")}
              title="Grid view"
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "list" ? "secondary" : "ghost"}
              size="icon"
              onClick={() => setSubscriptionsView("list")}
              title="List view"
            >
              <List className="h-4 w-4" />
            </Button>
          </div>
          <Button onClick={handleCreate} className="sm:w-auto w-full">
            <Plus className="mr-2 h-4 w-4" />
            <span className="hidden sm:inline">Add Subscription</span>
            <span className="sm:hidden">Add</span>
          </Button>
        </div>
      </div>

      {isLoading && subscriptions.length === 0 ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      ) : subscriptions.length === 0 ? (
        <Card>
          <CardContent className="flex h-64 flex-col items-center justify-center gap-4">
            <CreditCard className="h-12 w-12 text-muted-foreground" />
            <div className="text-center">
              <p className="text-lg font-medium">No subscriptions yet</p>
              <p className="text-sm text-muted-foreground">
                Click the button above to add your first subscription
              </p>
            </div>
            <Button onClick={handleCreate}>
              <Plus className="mr-2 h-4 w-4" />
              Add Subscription
            </Button>
          </CardContent>
        </Card>
      ) : filteredSubscriptions.length === 0 ? (
        <Card>
          <CardContent className="flex h-40 flex-col items-center justify-center gap-2">
            <p className="text-sm text-muted-foreground">No subscriptions match your filters</p>
            <Button variant="outline" size="sm" onClick={clearFilters}>Clear filters</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          <FilterSort
            filters={filters}
            sort={sort}
            onFiltersChange={setFilters}
            onSortChange={setSort}
            onClear={clearFilters}
            statusOptions={statusOptions}
            frequencyOptions={frequencyOptions}
            categories={categories}
            sortOptions={sortOptions}
          />
          <div className="text-sm text-muted-foreground">
            {filteredSubscriptions.length} subscription{filteredSubscriptions.length !== 1 ? "s" : ""}
          </div>
          {renderSubscriptions()}
        </div>
      )}

      <SubscriptionDialog
        key={editingSubscription?.id || "new"}
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        subscription={editingSubscription}
        defaultCurrency={user?.currency}
        onSubmit={handleSubmit}
      />

      <NotesEditDialog
        open={notesEditOpen}
        onOpenChange={setNotesEditOpen}
        notes={notesEditingSubscription?.notes || ""}
        onSave={handleNotesSave}
        title={`Notes: ${notesEditingSubscription?.name || ""}`}
      />
    </div>
  );
}
