import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { format } from "date-fns";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { CategoryCombobox } from "@/components/ui/combobox";
import { useCategoriesStore } from "@/stores/categoriesStore";
import type { Subscription, CreateSubscriptionRequest } from "@/lib/api/subscriptions";
import type { Category } from "@/lib/api/categories";

const frequencies = [
  { value: "weekly", label: "Weekly" },
  { value: "monthly", label: "Monthly" },
  { value: "quarterly", label: "Quarterly" },
  { value: "yearly", label: "Yearly" },
];

const currencies = [
  { value: "USD", label: "USD ($)" },
  { value: "EUR", label: "EUR (€)" },
  { value: "GBP", label: "GBP (£)" },
  { value: "JPY", label: "JPY (¥)" },
  { value: "CAD", label: "CAD ($)" },
  { value: "AUD", label: "AUD ($)" },
];

const subscriptionSchema = z.object({
  name: z.string().min(1, "Name is required"),
  amount: z.string().refine((val) => !isNaN(parseFloat(val)) && parseFloat(val) > 0, {
    message: "Amount must be a positive number",
  }),
  currency: z.string().min(3, "Select a currency"),
  frequency: z.string().min(1, "Select a frequency"),
  start_date: z.string().min(1, "Start date is required"),
  category_id: z.string().optional(),
  notes: z.string().optional(),
});

type SubscriptionFormData = z.infer<typeof subscriptionSchema>;

interface SubscriptionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  subscription?: Subscription;
  defaultDate?: string;
  onSubmit: (data: CreateSubscriptionRequest) => Promise<void>;
}

export function SubscriptionDialog({
  open,
  onOpenChange,
  subscription,
  defaultDate,
  onSubmit,
}: SubscriptionDialogProps) {
  const { categories, fetchCategories, createCategory } = useCategoriesStore();

  useEffect(() => {
    if (open) {
      fetchCategories();
    }
  }, [open, fetchCategories]);

  const {
    register,
    handleSubmit,
    reset,
    setValue,
    watch,
    formState: { errors, isSubmitting },
  } = useForm<SubscriptionFormData>({
    resolver: zodResolver(subscriptionSchema),
    defaultValues: {
      name: "",
      amount: "",
      currency: "USD",
      frequency: "monthly",
      start_date: format(new Date(), "yyyy-MM-dd"),
      category_id: "",
      notes: "",
    },
  });

  // Populate form when subscription or defaultDate changes (edit mode or new from calendar)
  useEffect(() => {
    if (subscription) {
      reset({
        name: subscription.name,
        amount: subscription.amount.toString(),
        currency: subscription.currency,
        frequency: subscription.frequency,
        start_date: subscription.start_date,
        category_id: subscription.category_id || "",
        notes: subscription.notes || "",
      });
    } else if (defaultDate) {
      // New subscription with pre-selected date from calendar
      reset({
        name: "",
        amount: "",
        currency: "USD",
        frequency: "monthly",
        start_date: defaultDate,
        category_id: "",
        notes: "",
      });
    } else {
      reset({
        name: "",
        amount: "",
        currency: "USD",
        frequency: "monthly",
        start_date: format(new Date(), "yyyy-MM-dd"),
        category_id: "",
        notes: "",
      });
    }
  }, [subscription, defaultDate, reset]);

  const categoryId = watch("category_id");

  const handleCategoryChange = (id: string) => {
    setValue("category_id", id);
  };

  const handleCreateCategory = async (name: string): Promise<Category> => {
    return await createCategory({ name });
  };

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      // Only reset if NOT editing (to avoid resetting during edit mode)
      if (!subscription) {
        reset();
      }
    }
    onOpenChange(newOpen);
  };

  const handleFormSubmit = async (data: SubscriptionFormData) => {
    const submitData: CreateSubscriptionRequest = {
      name: data.name,
      amount: parseFloat(data.amount),
      currency: data.currency,
      frequency: data.frequency,
      start_date: data.start_date,
      category_id: data.category_id || undefined,
      notes: data.notes,
    };

    await onSubmit(submitData);
    reset();
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]" onOpenAutoFocus={(e) => e.preventDefault()}>
        <DialogHeader>
          <DialogTitle>
            {subscription ? "Edit Subscription" : "Add Subscription"}
          </DialogTitle>
          <DialogDescription>
            {subscription
              ? "Make changes to your subscription here."
              : "Add a new recurring expense or subscription."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              placeholder="Netflix, Spotify, etc."
              onFocus={(e) => setTimeout(() => e.target.setSelectionRange(e.target.value.length, e.target.value.length), 0)}
              {...register("name")}
            />
            {errors.name && (
              <p className="text-xs text-destructive">{errors.name.message}</p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                step="0.01"
                min="0"
                placeholder="9.99"
                onFocus={(e) => setTimeout(() => e.target.setSelectionRange(e.target.value.length, e.target.value.length), 0)}
                {...register("amount")}
              />
              {errors.amount && (
                <p className="text-xs text-destructive">{errors.amount.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="currency">Currency</Label>
              <Select
                value={watch("currency") || subscription?.currency || "USD"}
                onValueChange={(value) => setValue("currency", value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select currency" />
                </SelectTrigger>
                <SelectContent>
                  {currencies.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                      {c.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="frequency">Frequency</Label>
              <Select
                value={watch("frequency") || subscription?.frequency || "monthly"}
                onValueChange={(value) => setValue("frequency", value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select frequency" />
                </SelectTrigger>
                <SelectContent>
                  {frequencies.map((f) => (
                    <SelectItem key={f.value} value={f.value}>
                      {f.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="start_date">Start Date</Label>
              <Input
                id="start_date"
                type="date"
                {...register("start_date")}
              />
              {errors.start_date && (
                <p className="text-xs text-destructive">{errors.start_date.message}</p>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>Category</Label>
            <CategoryCombobox
              value={categoryId || ""}
              onChange={handleCategoryChange}
              categories={categories}
              onCreateCategory={handleCreateCategory}
              placeholder="Search or create category..."
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="notes">Notes (optional)</Label>
            <Input
              id="notes"
              placeholder="Additional notes..."
              {...register("notes")}
            />
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting
                ? "Saving..."
                : subscription
                ? "Save Changes"
                : "Add Subscription"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
