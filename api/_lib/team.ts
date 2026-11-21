import type { RosterEntry, TeamResponse } from "@lol/shared";
import { NotFound, db, windowStart } from "./http.js";
import { storedImage } from "./images.js";
import { toRumor } from "./rumors.js";

export async function loadTeam(requested: string): Promise<TeamResponse> {
  const seasonStart = windowStart("season");
  const sql = db();
  const [team] = await sql`select name, short, league, region, logo_url from teams
                           where lower(name) = lower(${requested})
                           order by name = ${requested} desc
                           limit 1`;
  if (!team) throw new NotFound(`Team ${requested} not found`);
  const name: string = team.name;

  // The previous roster is the team's last tournament roster before the season started; the
  // current one applies every later move to it.
  const [[base] = [], current = [], rumors = []] = await sql.transaction(
    [
      sql`select tournament, roster,
                 (select coalesce(jsonb_object_agg(p.name, p.image_url), '{}')
                  from players p
                  where p.image_url is not null
                    and p.name in (select r->>'player' from jsonb_array_elements(roster) r)) as images
          from team_seasons
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
          select distinct on (members.player) members.player, members.role, p.image_url
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
          left join players p on p.name = members.player
          order by members.player, members.priority`,
      sql`select id, status, player, from_team, to_team, role, kind,
                 reported_at::text as reported_at, reliability, sources, notes,
                 transfer_id::text as transfer_id
          from rumors
          where lower(${name}) in (lower(from_team), lower(to_team))
          order by reported_at desc, id`,
    ],
    { readOnly: true },
  );

  const imageOf = new Map<string, string | null>([
    ...Object.entries<string>(base?.images ?? {}),
    ...current.map((row): [string, string | null] => [row.player, row.image_url]),
  ]);
  const toEntry = ({ player, role }: Record<string, any>): RosterEntry => ({
    player,
    role,
    imageUrl: storedImage(imageOf.get(player) ?? null, 96),
  });

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
      logoUrl: storedImage(team.logo_url, 192),
    },
    seasonStart,
    previousTournament: base?.tournament ?? null,
    roster,
    previousRoster,
    arrivals: [...now].filter((player) => !before.has(player)),
    departures: [...before].filter((player) => !now.has(player)),
    rumors: rumors.map(toRumor),
  };
}
