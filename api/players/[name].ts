import type { PlayerResponse } from "@lol/shared";
import { toFeedItem } from "../_lib/feed.js";
import { storedImage, teamLogo } from "../_lib/images.js";
import { NotFound, db, handler, pathName } from "../_lib/http.js";

export const GET = handler(async (url): Promise<PlayerResponse> => {
  const name = pathName(url);
  const sql = db();

  const [players = [], [latestMove] = [], [lastRoster] = [], feed = []] = await sql.transaction(
    [
      sql`select name, real_name, country, role, image_url from players where name = ${name}`,
      sql`select to_team, date::text as date,
                 (select short from teams where name = to_team) as to_short
          from transfers
          where player = ${name}
          order by date desc, id desc
          limit 1`,
      sql`select s.team, s.date_start::text as date_start,
                 (select short from teams where name = s.team) as short
          from team_seasons s
          where s.roster @> jsonb_build_array(jsonb_build_object('player', ${name}::text))
          order by s.date_start desc
          limit 1`,
      sql`select id, status, player, from_team, to_team, from_league, to_league, role, kind,
                 date::text as date, reliability, sources, notes, rumor_id, transfer_id,
                 (select short from teams where name = from_team) as from_short,
                 (select short from teams where name = to_team) as to_short,
                 (select image_url from players where name = ${name}) as player_image
          from market_feed
          where player = ${name}
          order by date desc, id desc`,
    ],
    { readOnly: true },
  );

  const [player] = players;
  if (!player) throw new NotFound(`Player ${name} not found`);

  // Moves only go back two seasons, so a player who has not moved since is placed by the
  // latest tournament roster that lists them, unless a later move overrides it.
  const fromRoster = lastRoster && (!latestMove || lastRoster.date_start > latestMove.date);
  const items = feed.map(toFeedItem);

  const currentTeam: string | null = fromRoster ? lastRoster.team : (latestMove?.to_team ?? null);

  return {
    player: {
      name: player.name,
      realName: player.real_name,
      country: player.country,
      role: player.role,
      imageUrl: storedImage(player.image_url, 320),
    },
    currentTeam,
    currentTeamLogo: teamLogo(currentTeam),
    currentTeamShort: fromRoster ? lastRoster.short : (latestMove?.to_short ?? null),
    timeline: items.filter((item) => item.status === "official"),
    rumors: items.filter((item) => item.status !== "official"),
  };
});
