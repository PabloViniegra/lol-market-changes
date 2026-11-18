import { neon } from "@neondatabase/serverless";
import type { ErrorResponse, StatsWindow } from "@lol/shared";

export class BadRequest extends Error {}
export class NotFound extends Error {}

export function db() {
  const databaseUrl = process.env.DATABASE_URL;
  if (!databaseUrl) throw new Error("DATABASE_URL is not configured");
  return neon(databaseUrl);
}

export function handler(run: (url: URL) => Promise<unknown>) {
  return async (request: Request): Promise<Response> => {
    try {
      const body = await run(new URL(request.url));
      return Response.json(body, {
        headers: { "Cache-Control": "s-maxage=300, stale-while-revalidate=600" },
      });
    } catch (error) {
      if (error instanceof BadRequest) return fail(400, error.message);
      if (error instanceof NotFound) return fail(404, error.message);
      console.error(error);
      return fail(500, "Internal error");
    }
  };
}

function fail(status: number, error: string): Response {
  const body: ErrorResponse = { error };
  return Response.json(body, { status });
}

export function oneOf<T extends string>(
  params: URLSearchParams,
  name: string,
  allowed: readonly T[],
): T | null {
  const value = params.get(name);
  if (value === null || value === "") return null;
  if (!allowed.includes(value as T)) {
    throw new BadRequest(`${name} must be one of: ${allowed.join(", ")}`);
  }
  return value as T;
}

export function text(params: URLSearchParams, name: string): string | null {
  const value = params.get(name)?.trim();
  if (!value) return null;
  if (value.length > 100) throw new BadRequest(`${name} is too long`);
  return value;
}

export function isoDate(params: URLSearchParams, name: string): string | null {
  const value = params.get(name);
  if (value === null || value === "") return null;
  if (!isIsoDate(value)) throw new BadRequest(`${name} must be a YYYY-MM-DD date`);
  return value;
}

export function isIsoDate(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(value) && toIsoDate(new Date(value)) === value;
}

function toIsoDate(date: Date): string {
  return Number.isNaN(date.getTime()) ? "" : date.toISOString().slice(0, 10);
}

/** Last segment of the path, for `api/<resource>/[name].ts` routes. */
export function pathName(url: URL): string {
  let name: string;
  try {
    name = decodeURIComponent(url.pathname.split("/").pop() ?? "");
  } catch {
    throw new BadRequest("name is not a valid URL component");
  }
  if (!name) throw new BadRequest("name is required");
  return name;
}

export const WINDOWS = ["7d", "30d", "season"] as const satisfies readonly StatsWindow[];

// The LoL season turns over after Worlds, so the offseason starts each November 1.
export function windowStart(window: StatsWindow, today = new Date()): string {
  if (window === "season") {
    const year = today.getUTCMonth() >= 10 ? today.getUTCFullYear() : today.getUTCFullYear() - 1;
    return `${year}-11-01`;
  }
  const days = window === "7d" ? 7 : 30;
  return toIsoDate(new Date(today.getTime() - days * 86_400_000));
}
