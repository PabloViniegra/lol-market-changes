import type { PlayerResponse } from "@lol/shared";
import { NotFound, db, handler, pathName } from "../_lib/http.js";
import { toRumor } from "../_lib/rumors.js";

export const GET = handler(async (url): Promise<PlayerResponse> => {
  const name = pathName(url);
  const sql = db();

  const [players = [], timeline = [], rumors = []] = await sql.transaction(
    [
      sql`select name, real_name, country, role, image_url from players where name = ${name}`,
      sql`select id::text as id, date::text as date, kind, from_team, to_team, role
          from transfers
          where player = ${name}
          order by date desc, id desc`,
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

  return {
    player: {
      name: player.name,
      realName: player.real_name,
      country: player.country,
      role: player.role,
      imageUrl: player.image_url,
    },
    currentTeam: timeline[0]?.to_team ?? null,
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
