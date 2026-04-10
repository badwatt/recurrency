import { useEffect, useCallback, useState } from "react";
import { toast } from "sonner";
import { Plus, Pencil, Trash2, LayoutGrid, List, CreditCard, Tag, StickyNote } from "lucide-react";
import * as LucideIcons from "lucide-react";
import type { LucideIcon } from "lucide-react";

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
import type { Subscription, CreateSubscriptionRequest } from "@/lib/api/subscriptions";

type ViewMode = "grid" | "list";

// Safe toast wrapper
const showToast = (type: "success" | "error", message: string) => {
  setTimeout(() => {
    if (type === "success") {
      toast.success(message);
    } else {
      toast.error(message);
    }
  }, 0);
};

// Helper to get Lucide icon component from kebab-case name
const getIconComponent = (iconName: string): LucideIcon => {
  if (!iconName) return Tag;
  const icons = LucideIcons as unknown as Record<string, LucideIcon>;
  const pascalCase = iconName
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
  return icons[pascalCase] || Tag;
};

interface SubscriptionCardProps {
  subscription: Subscription;
  viewMode: ViewMode;
  onEdit: (subscription: Subscription) => void;
  onDelete: (subscription: Subscription) => void;
  onNotesClick: (subscription: Subscription) => void;
  formatCurrency: (amount: number, currency: string) => string;
  getCategoryStyle: (categoryId: string) => { color: string; icon: string; name: string };
}

function SubscriptionCard({
  subscription,
  viewMode,
  onEdit,
  onDelete,
  onNotesClick,
  formatCurrency,
  getCategoryStyle,
}: SubscriptionCardProps) {
  const { color, icon, name: catName } = getCategoryStyle(subscription.category_id);
  const IconComponent = getIconComponent(icon);

  return (
    <Card className={!subscription.is_active ? "opacity-60" : "min-w-0"}>
      <CardContent className="p-3 sm:p-4">
        <div className="flex items-center gap-2 sm:gap-4">
          <div className="flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden">
            {subscription.logo_url ? (
              <img
                src={subscription.logo_url}
                alt={subscription.name}
                className="h-full w-full object-contain p-1"
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
            <div className="flex items-center gap-0.5 sm:gap-1">
              <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => onEdit(subscription)}>
                <Pencil className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => onDelete(subscription)}>
                <Trash2 className="h-4 w-4 text-destructive" />
              </Button>
            </div>
          </div>
        </div>
        {viewMode === "grid" && (
          <div className="mt-2 sm:hidden flex items-center gap-1">
            {subscription.notes && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onNotesClick(subscription)}>
                      <StickyNote className="h-3.5 w-3.5 text-muted-foreground" />
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
              className="font-normal text-xs"
              style={{
                color: color || "#6b7280",
                backgroundColor: `${color}15` || "#f3f4f6",
              }}
            >
              <IconComponent className="h-3 w-3 mr-1" />
              {catName}
            </Badge>
          </div>
        )}
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

  const activeSubscriptions = subscriptions.filter((s) => s.is_active);
  const inactiveSubscriptions = subscriptions.filter((s) => !s.is_active);

  const formatCurrency = (amount: number, currency: string) => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency,
    }).format(amount);
  };

  const renderSubscriptions = (subs: Subscription[]) => {
    if (subs.length === 0) return null;

    if (viewMode === "grid") {
      return (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {subs.map((subscription) => (
            <SubscriptionCard
              key={subscription.id}
              subscription={subscription}
              viewMode={viewMode}
              onEdit={handleEdit}
              onDelete={handleDelete}
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
        {subs.map((subscription) => (
          <SubscriptionCard
            key={subscription.id}
            subscription={subscription}
            viewMode={viewMode}
            onEdit={handleEdit}
            onDelete={handleDelete}
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Subscriptions</h1>
          <p className="text-muted-foreground">
            Manage your recurring expenses
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex border rounded-md">
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
          <Button onClick={handleCreate}>
            <Plus className="mr-2 h-4 w-4" />
            Add
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
      ) : (
        <div className="space-y-6">
          {activeSubscriptions.length > 0 && (
            <div className="space-y-4">
              <h2 className="text-lg font-semibold">
                Active ({activeSubscriptions.length})
              </h2>
              {renderSubscriptions(activeSubscriptions)}
            </div>
          )}

          {inactiveSubscriptions.length > 0 && (
            <div className="space-y-4">
              <h2 className="text-lg font-semibold text-muted-foreground">
                Inactive ({inactiveSubscriptions.length})
              </h2>
              {renderSubscriptions(inactiveSubscriptions)}
            </div>
          )}
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
