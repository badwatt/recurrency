import { useState, useRef, useEffect } from "react";
import { ChevronDown, Plus, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { getIcon } from "@/lib/icons";



interface CategoryOption {
  id: string;
  name: string;
  icon: string | null;
  color: string | null;
}

interface CategoryComboboxProps {
  value: string;
  onChange: (value: string) => void;
  categories: CategoryOption[];
  onCreateCategory: (name: string) => Promise<CategoryOption>;
  placeholder?: string;
}

export function CategoryCombobox({
  value,
  onChange,
  categories,
  onCreateCategory,
  placeholder = "Select category",
}: CategoryComboboxProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const selectedCategory = categories.find((c) => c.id === value);

  const filteredCategories = categories.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  );

  const showCreateOption = search.length > 0 && !filteredCategories.some(
    (c) => c.name.toLowerCase() === search.toLowerCase()
  );

  useEffect(() => {
    if (open && inputRef.current) {
      inputRef.current.focus();
    }
  }, [open]);

  const handleCreate = async () => {
    if (!search.trim()) return;
    
    setLoading(true);
    try {
      const newCategory = await onCreateCategory(search.trim());
      onChange(newCategory.id);
      setSearch("");
      setOpen(false);
    } catch (error) {
      console.error("Failed to create category:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (id: string) => {
    onChange(id);
    setSearch("");
    setOpen(false);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-full justify-between font-normal"
        >
          {selectedCategory ? (
            <div className="flex items-center gap-2 truncate">
              {(() => {
                const Icon = getIcon(selectedCategory.icon);
                return <Icon className="h-4 w-4 shrink-0" style={{ color: selectedCategory.color || "#6b7280" }} />;
              })()}
              <span className="truncate">{selectedCategory.name}</span>
            </div>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
          <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[300px] p-0" align="start">
        <div className="flex items-center border-b px-3">
          <Input
            ref={inputRef}
            placeholder="Search or create category..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="border-0 focus-visible:ring-0 h-9"
          />
        </div>
        <div className="max-h-[200px] overflow-y-auto py-1">
          {filteredCategories.length === 0 && !showCreateOption ? (
            <div className="py-6 text-center text-sm text-muted-foreground">
              No categories found
            </div>
          ) : (
            <>
              {filteredCategories.map((category) => {
                const Icon = getIcon(category.icon);
                return (
                  <button
                    key={category.id}
                    onClick={() => handleSelect(category.id)}
                    className={cn(
                      "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent",
                      value === category.id && "bg-accent"
                    )}
                  >
                    <Icon className="h-4 w-4 shrink-0" style={{ color: category.color || "#6b7280" }} />
                    <span className="flex-1 text-left truncate">{category.name}</span>
                    {value === category.id && (
                      <Check className="h-4 w-4 shrink-0 text-primary" />
                    )}
                  </button>
                );
              })}
              {showCreateOption && (
                <button
                  onClick={handleCreate}
                  disabled={loading}
                  className="relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent text-primary"
                >
                  <Plus className="h-4 w-4" />
                  <span>Create "{search}"</span>
                </button>
              )}
            </>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
