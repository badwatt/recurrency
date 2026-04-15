import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

export interface LogoResult {
  url: string;
  thumbnail: string;
  width?: number;
  height?: number;
}

export const logosApi = {
  async search(query: string): Promise<LogoResult[]> {
    const { token } = await getAuthParams();
    const response = await apiClient.get<{ results: LogoResult[] }>(
      `/api/logos/search?token=${encodeURIComponent(token)}&q=${encodeURIComponent(query)}`
    );
    return response.results;
  },
};
