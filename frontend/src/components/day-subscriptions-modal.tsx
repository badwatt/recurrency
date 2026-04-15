import { useState } from "react";
import { Pencil, Trash2, Plus, CreditCard } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { CalendarItem } from "@/lib/api/calendar";
import { handleImgError } from "@/lib/utils";

interface DaySubscriptionsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  date: string;
  items: CalendarItem[];
  onEdit: (subscriptionId: string) => void;
  onDelete: (subscriptionId: string, name: string) => void;
  onAdd: () => void;
  formatCurrency: (amount: string, currency: string) => string;
}

export function DaySubscriptionsModal({
  open,
  onOpenChange,
  date,
  items,
  onEdit,
  onDelete,
  onAdd,
  formatCurrency,
}: DaySubscriptionsModalProps) {
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const formattedDate = new Date(date).toLocaleDateString("en-US", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  const totalAmount = items.reduce(
    (sum, item) => sum + parseFloat(item.amount),
    0
  );

  const handleDelete = (subscriptionId: string, name: string) => {
    setDeletingId(subscriptionId);
    onDelete(subscriptionId, name);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[480px]" onOpenAutoFocus={(e) => e.preventDefault()}>
        <DialogHeader className="pr-8">
          <DialogTitle className="flex items-center justify-between gap-4">
            <span>{formattedDate}</span>
            <span className="text-lg font-semibold shrink-0">
              {items.length > 0 && formatCurrency(totalAmount.toString(), items[0].currency)}
            </span>
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-3 max-h-[400px] overflow-y-auto">
          {items.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <CreditCard className="h-12 w-12 mx-auto mb-3 opacity-50" />
              <p>No subscriptions scheduled for this day</p>
            </div>
          ) : (
            items.map((item) => (
              <div
                key={item.subscription_id}
                className="flex items-center gap-3 p-3 rounded-lg border bg-card"
              >
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-primary/10 overflow-hidden">
                  {item.logo_url ? (
                    <img
                      src={item.logo_url}
                      alt={item.name}
                      className="h-full w-full object-contain p-1"
                      onError={handleImgError}
                    />
                  ) : (
                    <CreditCard className="h-5 w-5 text-primary" />
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium truncate">{item.name}</p>
                  <p className="text-sm text-muted-foreground">
                    {item.frequency} • {formatCurrency(item.amount, item.currency)}
                  </p>
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={() => onEdit(item.subscription_id)}
                    disabled={deletingId === item.subscription_id}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-destructive"
                    onClick={() => handleDelete(item.subscription_id, item.name)}
                    disabled={deletingId === item.subscription_id}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="flex justify-end pt-4 border-t">
          <Button onClick={onAdd}>
            <Plus className="h-4 w-4 mr-2" />
            Add Subscription
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
