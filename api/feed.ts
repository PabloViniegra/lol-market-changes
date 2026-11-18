import type { FeedItem, FeedResponse, FeedStatus, League, MoveKind } from "@lol/shared";
import { BadRequest, db, handler, isIsoDate, isoDate, oneOf, text } from "./_lib/http.js";

const LEAGUES = ["LCK", "LPL", "LEC", "LCS"] as const satisfies readonly League[];
const KINDS = [
  "transfer",
  "free_agent_signing",
  "release",
  "loan",
  "promotion",
  "bench",
  "sub",
  "retirement",
] as const satisfies readonly MoveKind[];
const STATUSES = [
  "official",
  "rumor",
  "confirmed",
  "denied",
  "stale",
] as const satisfies readonly FeedStatus[];
// Confirmed rumors show up through their official move; denied and stale ones are opt-in.
const DEFAULT_STATUSES: FeedStatus[] = ["official", "rumor"];
const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 100;

export const GET = handler(async (url): Promise<FeedResponse> => {
  const params = url.searchParams;
  const league = oneOf(params, "league", LEAGUES);
  const team = text(params, "team");
  const role = text(params, "role");
  const kind = oneOf(params, "kind", KINDS);
  const statuses = parseStatuses(params.get("status"));
  const from = isoDate(params, "from");
  const to = isoDate(params, "to");
  const limit = parseLimit(params.get("limit"));
  const cursor = parseCursor(params.get("cursor"));

  const rows = await db()`
    select id, status, player, from_team, to_team, from_league, to_league, role, kind,
           date::text as date, reliability, sources, notes, rumor_id, transfer_id,
           (select short from teams where name = from_team) as from_short,
           (select short from teams where name = to_team) as to_short
    from market_feed
    where status = any(${statuses}::text[])
      and (${league}::text is null or ${league}::text in (from_league, to_league))
      and (${team}::text is null or ${team}::text in (from_team, to_team))
      and (${role}::text is null or role = ${role}::text)
      and (${kind}::text is null or kind = ${kind}::text)
      and (${from}::date is null or date >= ${from}::date)
      and (${to}::date is null or date <= ${to}::date)
      and (${cursor?.date ?? null}::date is null
           or (date, id) < (${cursor?.date ?? null}::date, ${cursor?.id ?? null}::text))
    order by date desc, id desc
    limit ${limit + 1}`;

  const items: FeedItem[] = rows.slice(0, limit).map((row) => ({
    id: row.id,
    status: row.status,
    player: row.player,
    fromTeam: row.from_team,
    toTeam: row.to_team,
    fromShort: row.from_short,
    toShort: row.to_short,
    fromLeague: row.from_league,
    toLeague: row.to_league,
    role: row.role,
    kind: row.kind,
    date: row.date,
    reliability: row.reliability,
    sources: row.sources,
    notes: row.notes,
    rumorId: row.rumor_id,
    transferId: row.transfer_id,
  }));
  const last = items.at(-1);
  const nextCursor = rows.length > limit && last ? encodeCursor(last.date, last.id) : null;
  return { items, nextCursor };
});

function parseStatuses(value: string | null): FeedStatus[] {
  if (!value) return DEFAULT_STATUSES;
  const statuses = value.split(",").map((s) => s.trim());
  const invalid = statuses.find((s) => !(STATUSES as readonly string[]).includes(s));
  if (invalid !== undefined) {
    throw new BadRequest(`status must be a comma-separated list of: ${STATUSES.join(", ")}`);
  }
  return statuses as FeedStatus[];
}

function parseLimit(value: string | null): number {
  if (value === null || value === "") return DEFAULT_LIMIT;
  const limit = Number(value);
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_LIMIT) {
    throw new BadRequest(`limit must be an integer between 1 and ${MAX_LIMIT}`);
  }
  return limit;
}

function encodeCursor(date: string, id: string): string {
  return Buffer.from(JSON.stringify([date, id])).toString("base64url");
}

function parseCursor(value: string | null): { date: string; id: string } | null {
  if (!value) return null;
  let parsed: unknown;
  try {
    parsed = JSON.parse(Buffer.from(value, "base64url").toString());
  } catch {
    throw new BadRequest("cursor is invalid");
  }
  const [date, id] = Array.isArray(parsed) ? parsed : [];
  if (typeof date !== "string" || !isIsoDate(date) || typeof id !== "string") {
    throw new BadRequest("cursor is invalid");
  }
  return { date, id };
}
