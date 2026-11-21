import type { FeedItem } from "@lol/shared";
import { storedImage } from "./images.js";

export function toFeedItem(row: Record<string, any>): FeedItem {
  return {
    id: row.id,
    status: row.status,
    player: row.player,
    fromTeam: row.from_team,
    toTeam: row.to_team,
    fromShort: row.from_short,
    toShort: row.to_short,
    fromLogo: storedImage(row.from_logo ?? null, 64),
    toLogo: storedImage(row.to_logo ?? null, 64),
    playerImage: storedImage(row.player_image ?? null, 96),
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
  };
}
