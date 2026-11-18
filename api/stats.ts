import type { StatsResponse } from "@lol/shared";
import { WINDOWS, db, handler, oneOf, windowStart } from "./_lib/http.js";

export const GET = handler(async (url): Promise<StatsResponse> => {
  const window = oneOf(url.searchParams, "window", WINDOWS) ?? "season";
  const since = windowStart(window);
  const sql = db();

  const [
    [totals] = [],
    byLeagueAndKind = [],
    [freeAgents] = [],
    topTeamsByChurn = [],
    [rumors] = [],
  ] = await sql.transaction(
    [
      sql`select count(*)::int as total
            from market_feed where status = 'official' and date >= ${since}::date`,
      sql`select l.league, m.kind, count(*)::int as count
            from market_feed m
            cross join lateral (
              select distinct league from unnest(array[m.from_league, m.to_league]) as league
              where league is not null
            ) l
            where m.status = 'official' and m.date >= ${since}::date
            group by 1, 2
            order by 1, 2`,
      sql`select count(*)::int as count
            from (
              select distinct on (player) id, kind, date
              from transfers
              order by player, date desc, id desc
            ) latest
            where kind = 'release' and date >= ${since}::date
              and exists (select 1 from market_feed f where f.id = latest.id::text)`,
      sql`select c.team, t.league, sum(c.arrival)::int as arrivals,
                   sum(1 - c.arrival)::int as departures
            from (
              select to_team as team, 1 as arrival from market_feed
              where status = 'official' and date >= ${since}::date and to_league is not null
                and to_team is distinct from from_team
              union all
              select from_team, 0 from market_feed
              where status = 'official' and date >= ${since}::date and from_league is not null
                and from_team is distinct from to_team
            ) c
            left join teams t on t.name = c.team
            group by c.team, t.league
            order by count(*) desc, c.team
            limit 10`,
      sql`select count(*) filter (where status = 'confirmed')::int as confirmed,
                   count(*) filter (where status in ('confirmed', 'denied', 'stale'))::int
                     as resolved
            from rumors where reported_at >= ${since}::date`,
    ],
    { readOnly: true },
  );

  return {
    window,
    since,
    totalMoves: totals?.total ?? 0,
    byLeagueAndKind: byLeagueAndKind.map(({ league, kind, count }) => ({ league, kind, count })),
    freeAgents: freeAgents?.count ?? 0,
    topTeamsByChurn: topTeamsByChurn.map(({ team, league, arrivals, departures }) => ({
      team,
      league,
      arrivals,
      departures,
    })),
    rumorConfirmationRate: rumors?.resolved ? rumors.confirmed / rumors.resolved : null,
  };
});
