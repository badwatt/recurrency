import { useState } from "react";
import { Search, X, Image as ImageIcon, Globe } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { logosApi, type LogoResult } from "@/lib/api/logos";
import { handleImgError } from "@/lib/utils";
import { toast } from "sonner";

interface LogoSearchProps {
  query: string;
  onSelect: (logoUrl: string) => void;
  selectedLogo?: string | null;
}

export function LogoSearch({ query, onSelect, selectedLogo }: LogoSearchProps) {
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState(query);
  const [loading, setLoading] = useState(false);
  const [logos, setLogos] = useState<LogoResult[]>([]);

  const searchLogos = async () => {
    if (!searchQuery.trim()) return;
    
    setLoading(true);
    try {
      const results = await logosApi.search(searchQuery);
      setLogos(results);
      if (results.length === 0) {
        toast.info("No logos found. Try a different search term.");
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to search logos");
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (logoUrl: string) => {
    onSelect(logoUrl);
    setOpen(false);
  };

  const handleClear = () => {
    onSelect("");
  };

  const handleOpen = () => {
    setSearchQuery(query);
    setLogos([]);
    setOpen(true);
    // Auto-search if we have a query
    if (query) {
      setTimeout(searchLogos, 0);
    }
  };

  return (
    <>
      <div className="flex items-center gap-2">
        {selectedLogo ? (
          <div className="flex items-center gap-2">
            <img
              src={selectedLogo}
              alt="Selected logo"
              className="h-8 w-8 object-contain rounded border"
              onError={handleImgError}
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={handleClear}
              title="Clear logo"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ) : null}
        
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleOpen}
        >
          <ImageIcon className="h-4 w-4 mr-1" />
          {selectedLogo ? "Change" : "Add logo"}
        </Button>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="sm:max-w-[480px]" onOpenAutoFocus={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle>Search Logo</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="flex gap-2">
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Enter company name or domain..."
                onKeyDown={(e) => e.key === "Enter" && searchLogos()}
              />
              <Button
                type="button"
                onClick={searchLogos}
                disabled={loading || !searchQuery.trim()}
              >
                <Search className="h-4 w-4 mr-1" />
                Search
              </Button>
            </div>

            {loading && (
              <div className="flex h-32 items-center justify-center">
                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              </div>
            )}

            {!loading && logos.length === 0 && searchQuery && (
              <div className="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Globe className="h-8 w-8" />
                <p className="text-sm">No logos found</p>
                <p className="text-xs">Try searching for the company website (e.g., spotify.com)</p>
              </div>
            )}

            {logos.length > 0 && (
              <div className="grid grid-cols-4 gap-3">
                {logos.map((logo) => (
                  <button
                    key={logo.url}
                    type="button"
                    onClick={() => handleSelect(logo.url)}
                    className="flex aspect-square items-center justify-center rounded-md border border-input bg-background p-2 transition-colors hover:border-primary hover:bg-muted/50"
                    title={logo.url}
                  >
                    <img
                      src={logo.thumbnail}
                      alt={`Logo ${logo.url}`}
                      className="max-h-full max-w-full object-contain"
                      loading="lazy"
                      onError={handleImgError}
                    />
                  </button>
                ))}
              </div>
            )}

            {logos.length > 0 && (
              <p className="text-xs text-muted-foreground text-center">
                Click a logo to select it. Images from DuckDuckGo.
              </p>
            )}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
