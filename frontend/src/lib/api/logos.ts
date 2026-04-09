import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface LogoResult {
  url: string;
  thumbnail: string;
  width?: number;
  height?: number;
}

async function getAuthHeaders() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const logosApi = {
  async search(query: string): Promise<LogoResult[]> {
    const { token } = await getAuthHeaders();
    const response = await apiClient.get<{ results: LogoResult[] }>(
      `/api/logos/search?token=${encodeURIComponent(token)}&q=${encodeURIComponent(query)}`
    );
    return response.results;
  },
};
