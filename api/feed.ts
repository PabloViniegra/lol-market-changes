import type { FeedItem, FeedResponse, FeedStatus, League, MoveKind } from "@lol/shared";
import { toFeedItem } from "./_lib/feed.js";
import {
  BadRequest,
  WINDOWS,
  db,
  handler,
  isIsoDate,
  isoDate,
  oneOf,
  text,
  windowStart,
} from "./_lib/http.js";

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
  "contradicted",
] as const satisfies readonly FeedStatus[];
const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 100;

export const GET = handler(async (url): Promise<FeedResponse> => {
  const params = url.searchParams;
  const league = oneOf(params, "league", LEAGUES);
  const team = text(params, "team");
  const player = text(params, "player");
  const role = text(params, "role");
  const kind = oneOf(params, "kind", KINDS);
  const market = oneOf(params, "view", ["market"]) === "market";
  const statuses = parseStatuses(params.get("status"), market);
  const window = oneOf(params, "window", WINDOWS);
  const from = isoDate(params, "from") ?? (window ? windowStart(window) : null);
  const to = isoDate(params, "to");
  const limit = parseLimit(params.get("limit"));
  const cursor = parseCursor(params.get("cursor"));

  const rows = await db()`
    with candidates as not materialized (
      select *,
        case when ${market}::boolean and status = 'official' and rumor_id is not null
          then 'confirmed' else status end as display_status
      from market_feed
      where (${league}::text is null or ${league}::text in (from_league, to_league))
        and (${team}::text is null or lower(${team}::text) in (lower(from_team), lower(to_team)))
        and (${player}::text is null or strpos(lower(player), lower(${player}::text)) > 0)
        and (${role}::text is null or role = ${role}::text)
        and (${kind}::text is null or kind = ${kind}::text)
        and (${from}::date is null or date >= ${from}::date)
        and (${to}::date is null or date <= ${to}::date)
    ),
    matching as not materialized (
      select * from candidates where display_status = any(${statuses}::text[])
    )
    select id, display_status as status, player, from_team, to_team, from_league, to_league, role, kind,
           date::text as date, reliability, sources, notes, rumor_id, transfer_id,
           (select short from teams where name = from_team) as from_short,
           (select short from teams where name = to_team) as to_short,
           (select logo_url from teams where name = from_team) as from_logo,
           (select logo_url from teams where name = to_team) as to_logo,
           (select image_url from players where name = m.player) as player_image
    from matching m
    where (m.status <> 'confirmed' or not exists (
      select 1 from matching official
      where official.status = 'official' and official.id = m.transfer_id
    ))
      and (${cursor?.date ?? null}::date is null
           or (date, id) < (${cursor?.date ?? null}::date, ${cursor?.id ?? null}::text))
    order by date desc, id desc
    limit ${limit + 1}`;

  const items: FeedItem[] = rows.slice(0, limit).map(toFeedItem);
  const last = items.at(-1);
  const nextCursor = rows.length > limit && last ? encodeCursor(last.date, last.id) : null;
  return { items, nextCursor };
});

function parseStatuses(value: string | null, market: boolean): FeedStatus[] {
  if (!value) return market ? ["official", "confirmed", "rumor"] : ["official", "rumor"];
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
