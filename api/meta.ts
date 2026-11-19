import { neon } from "@neondatabase/serverless";
import type { MetaResponse, SyncStatus } from "@lol/shared";

const ATTRIBUTION = {
  text: "Official roster data from Leaguepedia, licensed CC BY-SA",
  url: "https://lol.fandom.com/",
};

export async function GET(): Promise<Response> {
  const databaseUrl = process.env.DATABASE_URL;
  if (!databaseUrl) {
    return Response.json({ error: "DATABASE_URL is not configured" }, { status: 500 });
  }

  const sql = neon(databaseUrl);
  // sync_state only exists once the ingestor migrations have run.
  const [table] = await sql`select to_regclass('public.sync_state') is not null as exists`;
  let lastRunAt: string | null = null;
  let lastSuccessfulRunAt: string | null = null;
  let lastRunStatus: SyncStatus = "never";
  if (table?.exists) {
    const [row] = await sql`
      select last_run_at, last_run_status,
        case when last_run_status = 'ok' then last_run_at
          else (to_jsonb(s)->>'last_successful_run_at')::timestamptz
        end as last_successful_run_at
      from sync_state s where id = 1`;
    if (row) {
      lastRunAt = row.last_run_at ? new Date(row.last_run_at).toISOString() : null;
      lastSuccessfulRunAt = row.last_successful_run_at
        ? new Date(row.last_successful_run_at).toISOString()
        : null;
      lastRunStatus = row.last_run_status ?? "never";
    }
  }

  const body: MetaResponse = {
    lastRunAt,
    lastSuccessfulRunAt,
    lastRunStatus,
    attribution: ATTRIBUTION,
  };
  return Response.json(body, {
    headers: { "Cache-Control": "s-maxage=300, stale-while-revalidate=600" },
  });
}
