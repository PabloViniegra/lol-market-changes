import type { FlowsResponse } from "@lol/shared";
import { WINDOWS, db, handler, oneOf, windowStart } from "./_lib/http.js";

const LEVELS = ["league", "team"] as const;

export const GET = handler(async (url): Promise<FlowsResponse> => {
  const params = url.searchParams;
  const level = oneOf(params, "level", LEVELS) ?? "league";
  const window = oneOf(params, "window", WINDOWS) ?? "season";
  const since = windowStart(window);

  const rows = await db()`
    select source, target, count(*)::int as value
    from (
      select
        case when ${level}::text = 'league' then coalesce(from_league, 'Other') else from_team end
          as source,
        case when ${level}::text = 'league' then coalesce(to_league, 'Other') else to_team end
          as target
      from market_feed
      where status = 'official' and date >= ${since}::date
        and from_team is not null and to_team is not null and from_team <> to_team
    ) moves
    group by source, target
    order by value desc, source, target`;

  return {
    level,
    window,
    since,
    links: rows.map(({ source, target, value }) => ({ source, target, value })),
  };
});
