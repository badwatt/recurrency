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
import type { Transaction, CreateTransactionRequest } from "@/lib/api/transactions";
import type { Category } from "@/lib/api/categories";

const currencies = [
  { value: "USD", label: "USD ($)" },
  { value: "EUR", label: "EUR (€)" },
  { value: "GBP", label: "GBP (£)" },
  { value: "JPY", label: "JPY (¥)" },
  { value: "CAD", label: "CAD ($)" },
  { value: "AUD", label: "AUD ($)" },
];

const transactionSchema = z.object({
  name: z.string().min(1, "Name is required"),
  amount: z.string().refine((val) => !isNaN(parseFloat(val)) && parseFloat(val) > 0, {
    message: "Amount must be a positive number",
  }),
  currency: z.string().min(3, "Select a currency"),
  date: z.string().min(1, "Date is required"),
  category_id: z.string().optional(),
  notes: z.string().optional(),
});

type TransactionFormData = z.infer<typeof transactionSchema>;

interface TransactionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  transaction?: Transaction;
  onSubmit: (data: CreateTransactionRequest) => Promise<void>;
}

export function TransactionDialog({
  open,
  onOpenChange,
  transaction,
  onSubmit,
}: TransactionDialogProps) {
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
  } = useForm<TransactionFormData>({
    resolver: zodResolver(transactionSchema),
    defaultValues: {
      name: "",
      amount: "",
      currency: "USD",
      date: format(new Date(), "yyyy-MM-dd"),
      category_id: "",
      notes: "",
    },
  });

  // Populate form when transaction changes (edit mode)
  useEffect(() => {
    if (transaction) {
      reset({
        name: transaction.name,
        amount: transaction.amount.toString(),
        currency: transaction.currency,
        date: transaction.date,
        category_id: transaction.category_id || "",
        notes: transaction.notes || "",
      });
    }
  }, [transaction, reset]);

  const categoryId = watch("category_id");

  const handleCategoryChange = (id: string) => {
    setValue("category_id", id);
  };

  const handleCreateCategory = async (name: string): Promise<Category> => {
    return await createCategory({ name });
  };

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      if (!transaction) {
        reset();
      }
    }
    onOpenChange(newOpen);
  };

  const handleFormSubmit = async (data: TransactionFormData) => {
    const submitData: CreateTransactionRequest = {
      name: data.name,
      amount: parseFloat(data.amount),
      currency: data.currency,
      date: data.date,
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
            {transaction ? "Edit Transaction" : "Add Transaction"}
          </DialogTitle>
          <DialogDescription>
            {transaction
              ? "Make changes to your transaction here."
              : "Add a new one-time expense."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              placeholder="Coffee, Groceries, etc."
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
                placeholder="25.00"
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
                value={watch("currency") || transaction?.currency || "USD"}
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
              <Label htmlFor="date">Date</Label>
              <Input
                id="date"
                type="date"
                {...register("date")}
              />
              {errors.date && (
                <p className="text-xs text-destructive">{errors.date.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label>Category</Label>
              <CategoryCombobox
                value={categoryId || ""}
                onChange={handleCategoryChange}
                categories={categories}
                onCreateCategory={handleCreateCategory}
                placeholder="Search or create..."
              />
            </div>
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
                : transaction
                ? "Save Changes"
                : "Add Transaction"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
