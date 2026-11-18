import type { PlayerResponse } from "@lol/shared";
import { NotFound, db, handler, pathName } from "../_lib/http.js";
import { toRumor } from "../_lib/rumors.js";

export const GET = handler(async (url): Promise<PlayerResponse> => {
  const name = pathName(url);
  const sql = db();

  const [players = [], timeline = [], [lastRoster] = [], rumors = []] = await sql.transaction(
    [
      sql`select name, real_name, country, role, image_url from players where name = ${name}`,
      sql`select id::text as id, date::text as date, kind, from_team, to_team, role
          from transfers
          where player = ${name}
          order by date desc, id desc`,
      sql`select s.team, s.date_start::text as date_start
          from team_seasons s
          where s.roster @> jsonb_build_array(jsonb_build_object('player', ${name}::text))
          order by s.date_start desc
          limit 1`,
      sql`select id, status, player, from_team, to_team, role, kind,
                 reported_at::text as reported_at, reliability, sources, notes,
                 transfer_id::text as transfer_id
          from rumors
          where player = ${name}
          order by reported_at desc, id`,
    ],
    { readOnly: true },
  );

  const [player] = players;
  if (!player) throw new NotFound(`Player ${name} not found`);

  // Moves only go back two seasons, so a player who has not moved since is placed by the
  // latest tournament roster that lists them, unless a later move overrides it.
  const [latestMove] = timeline;
  const currentTeam =
    lastRoster && (!latestMove || lastRoster.date_start > latestMove.date)
      ? lastRoster.team
      : (latestMove?.to_team ?? null);

  return {
    player: {
      name: player.name,
      realName: player.real_name,
      country: player.country,
      role: player.role,
      imageUrl: player.image_url,
    },
    currentTeam,
    timeline: timeline.map(({ id, date, kind, from_team, to_team, role }) => ({
      id,
      date,
      kind,
      fromTeam: from_team,
      toTeam: to_team,
      role,
    })),
    rumors: rumors.map(toRumor),
  };
});
