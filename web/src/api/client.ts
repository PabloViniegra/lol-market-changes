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

export async function getJson<T>(path: string, params: QueryParams = {}): Promise<T> {
  const search = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value !== null && value !== undefined && value !== "") search.set(key, String(value));
  }

  const query = search.size > 0 ? `?${search}` : "";
  const response = await fetch(`${path}${query}`);

  if (!response.ok) throw new HttpError(response.status, path);

  return response.json();
}
