import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Badge } from "@/components/ui/badge";
import { ArrowUp, ArrowDown, X, Filter, ArrowUpDown } from "lucide-react";

export interface SortOption {
  value: string;
  label: string;
}

export interface FilterConfig {
  status?: string;
  frequency?: string;
  categoryId?: string;
  priceMin?: string;
  priceMax?: string;
}

export interface SortConfig {
  field: string;
  direction: "asc" | "desc";
}

interface FilterSortProps {
  filters: FilterConfig;
  sort: SortConfig;
  onFiltersChange: (filters: FilterConfig) => void;
  onSortChange: (sort: SortConfig) => void;
  onClear: () => void;
  statusOptions?: { value: string; label: string }[];
  frequencyOptions?: { value: string; label: string }[];
  categories: { id: string; name: string }[];
  sortOptions: SortOption[];
}

export function FilterSort({
  filters,
  sort,
  onFiltersChange,
  onSortChange,
  onClear,
  statusOptions,
  frequencyOptions,
  categories,
  sortOptions,
}: FilterSortProps) {
  const activeCount = countActiveFilters(filters);

  return (
    <div className="flex items-center gap-2 flex-wrap">
      {/* Filters button — opens popover */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-1.5">
            <Filter className="h-3.5 w-3.5" />
            Filters
            {activeCount > 0 && (
              <Badge variant="secondary" className="ml-1 h-4 px-1.5 text-[10px]">
                {activeCount}
              </Badge>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-80" align="start">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h4 className="font-medium text-sm">Filters</h4>
              {activeCount > 0 && (
                <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={onClear}>
                  Clear all
                </Button>
              )}
            </div>

            {statusOptions && (
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-muted-foreground">Status</label>
                <Select
                  value={filters.status || "all"}
                  onValueChange={(v) =>
                    onFiltersChange({ ...filters, status: v === "all" ? undefined : v })
                  }
                >
                  <SelectTrigger className="h-8">
                    <SelectValue placeholder="All" />
                  </SelectTrigger>
                  <SelectContent>
                    {statusOptions.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {frequencyOptions && (
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-muted-foreground">Frequency</label>
                <Select
                  value={filters.frequency || "all"}
                  onValueChange={(v) =>
                    onFiltersChange({ ...filters, frequency: v === "all" ? undefined : v })
                  }
                >
                  <SelectTrigger className="h-8">
                    <SelectValue placeholder="All" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All frequencies</SelectItem>
                    {frequencyOptions.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Category</label>
              <Select
                value={filters.categoryId || "all"}
                onValueChange={(v) =>
                  onFiltersChange({ ...filters, categoryId: v === "all" ? undefined : v })
                }
              >
                <SelectTrigger className="h-8">
                  <SelectValue placeholder="All" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All categories</SelectItem>
                  {categories.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Price range</label>
              <div className="flex items-center gap-2">
                <Input
                  type="number"
                  placeholder="Min"
                  className="h-8 text-sm"
                  value={filters.priceMin || ""}
                  onChange={(e) =>
                    onFiltersChange({ ...filters, priceMin: e.target.value || undefined })
                  }
                />
                <span className="text-muted-foreground text-xs">–</span>
                <Input
                  type="number"
                  placeholder="Max"
                  className="h-8 text-sm"
                  value={filters.priceMax || ""}
                  onChange={(e) =>
                    onFiltersChange({ ...filters, priceMax: e.target.value || undefined })
                  }
                />
              </div>
            </div>
          </div>
        </PopoverContent>
      </Popover>

      {activeCount > 0 && (
        <Button variant="ghost" size="sm" className="h-8 px-2" onClick={onClear}>
          <X className="h-3 w-3 mr-1" />
          Clear
        </Button>
      )}

      {/* Sort — separate dropdown */}
      <div className="flex items-center gap-1.5 ml-auto">
        <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground" />
        <Select
          value={sort.field}
          onValueChange={(v) => onSortChange({ ...sort, field: v })}
        >
          <SelectTrigger className="h-8 w-[150px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {sortOptions.map((o) => (
              <SelectItem key={o.value} value={o.value}>
                {o.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() =>
            onSortChange({ ...sort, direction: sort.direction === "asc" ? "desc" : "asc" })
          }
          title={sort.direction === "asc" ? "Ascending" : "Descending"}
        >
          {sort.direction === "asc" ? (
            <ArrowUp className="h-4 w-4" />
          ) : (
            <ArrowDown className="h-4 w-4" />
          )}
        </Button>
      </div>
    </div>
  );
}

function countActiveFilters(f: FilterConfig): number {
  let n = 0;
  if (f.status) n++;
  if (f.frequency) n++;
  if (f.categoryId) n++;
  if (f.priceMin) n++;
  if (f.priceMax) n++;
  return n;
}