import type { RosterEntry, TeamResponse } from "@lol/shared";
import { NotFound, db, handler, pathName, windowStart } from "../_lib/http.js";
import { toRumor } from "../_lib/rumors.js";

const toEntry = ({ player, role }: Record<string, any>): RosterEntry => ({ player, role });

export const GET = handler(async (url): Promise<TeamResponse> => {
  const name = pathName(url);
  const seasonStart = windowStart("season");
  const sql = db();

  // The previous roster is the team's last tournament roster before the season started; the
  // current one applies every later move to it.
  const [teams = [], [base] = [], current = [], rumors = []] = await sql.transaction(
    [
      sql`select name, short, league, region, logo_url from teams where name = ${name}`,
      sql`select tournament, roster from team_seasons
          where team = ${name} and date_start < ${seasonStart}::date and roster <> '[]'
          order by date_start desc limit 1`,
      sql`with base as (
            select date_start, roster from team_seasons
            where team = ${name} and date_start < ${seasonStart}::date and roster <> '[]'
            order by date_start desc limit 1
          ),
          latest as (
            select distinct on (player) player, to_team, role
            from transfers
            where date >= coalesce((select date_start from base), '-infinity'::date)
            order by player, date desc, id desc
          )
          select distinct on (player) player, role
          from (
            select r->>'player' as player, r->>'role' as role, 0 as priority
            from base, jsonb_array_elements(base.roster) r
            where not exists (
              select 1 from latest l
              where l.player = r->>'player' and l.to_team is distinct from ${name}
            )
            union all
            select player, role, 1 from latest where to_team = ${name}
          ) members
          order by player, priority`,
      sql`select id, status, player, from_team, to_team, role, kind,
                 reported_at::text as reported_at, reliability, sources, notes,
                 transfer_id::text as transfer_id
          from rumors
          where ${name} in (from_team, to_team)
          order by reported_at desc, id`,
    ],
    { readOnly: true },
  );

  const [team] = teams;
  if (!team) throw new NotFound(`Team ${name} not found`);
  const roster = current.map(toEntry);
  const previousRoster: RosterEntry[] = (base?.roster ?? []).map(toEntry);
  const now = new Set(roster.map((entry) => entry.player));
  const before = new Set(previousRoster.map((entry) => entry.player));

  return {
    team: {
      name: team.name,
      short: team.short,
      league: team.league,
      region: team.region,
      logoUrl: team.logo_url,
    },
    seasonStart,
    previousTournament: base?.tournament ?? null,
    roster,
    previousRoster,
    arrivals: [...now].filter((player) => !before.has(player)),
    departures: [...before].filter((player) => !now.has(player)),
    rumors: rumors.map(toRumor),
  };
});
