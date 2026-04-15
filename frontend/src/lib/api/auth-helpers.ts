import { auth } from "@/lib/auth";

export async function getAuthParams() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}