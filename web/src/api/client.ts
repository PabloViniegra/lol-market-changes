import { lastSyncedRun } from "./syncRefresh";

export type QueryParams = Record<string, string | number | null | undefined>;

export class HttpError extends Error {
  constructor(
    readonly status: number,
    path: string,
    method = "GET",
  ) {
    super(`${method} ${path} failed with ${status}`);
  }
}

export function isClientError(error: Error | null): boolean {
  return error instanceof HttpError && error.status < 500;
}

export async function getJson<T>(
  path: string,
  params: QueryParams = {},
  signal: AbortSignal | null = null,
): Promise<T> {
  const search = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value !== null && value !== undefined && value !== "") search.set(key, String(value));
  }

  const query = search.size > 0 ? `?${search}` : "";
  const response = await fetch(`${path}${query}`, { signal });

  if (!response.ok) throw new HttpError(response.status, path);

  return response.json();
}

// Data URLs change with each ingest so the CDN cannot answer a post-ingest refetch with an older copy.
export function getData<T>(
  path: string,
  params: QueryParams = {},
  signal: AbortSignal | null = null,
): Promise<T> {
  return getJson<T>(path, { ...params, sync: lastSyncedRun() }, signal);
}
