import { toast } from "sonner";

export function showToast(type: "success" | "error", message: string) {
  setTimeout(() => {
    toast[type](message);
  }, 0);
}