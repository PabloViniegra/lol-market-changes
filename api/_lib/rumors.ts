import type { Rumor } from "@lol/shared";

export function toRumor(row: Record<string, any>): Rumor {
  return {
    id: row.id,
    status: row.status,
    player: row.player,
    fromTeam: row.from_team,
    toTeam: row.to_team,
    role: row.role,
    kind: row.kind,
    reportedAt: row.reported_at,
    reliability: row.reliability,
    sources: row.sources,
    notes: row.notes,
    transferId: row.transfer_id,
  };
}
