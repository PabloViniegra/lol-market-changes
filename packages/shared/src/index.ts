export type SyncStatus = "ok" | "error" | "never";

export interface MetaResponse {
  lastRunAt: string | null;
  lastSuccessfulRunAt: string | null;
  lastRunStatus: SyncStatus;
  attribution: {
    text: string;
    url: string;
  };
}

export interface ErrorResponse {
  error: string;
}

export type League = "LCK" | "LPL" | "LEC" | "LCS";

export type MoveKind =
  | "transfer"
  | "free_agent_signing"
  | "release"
  | "loan"
  | "promotion"
  | "bench"
  | "sub"
  | "retirement";

export type RumorStatus = "rumor" | "confirmed" | "denied" | "stale";

export type FeedStatus = "official" | RumorStatus;

export type StatsWindow = "7d" | "30d" | "season";

export interface Source {
  url: string;
  outlet?: string | null;
}

export interface FeedItem {
  id: string;
  status: FeedStatus;
  player: string;
  fromTeam: string | null;
  toTeam: string | null;
  fromShort: string | null;
  toShort: string | null;
  fromLeague: League | null;
  toLeague: League | null;
  role: string | null;
  kind: string | null;
  /** YYYY-MM-DD: move date for official items, report date for rumors. */
  date: string;
  reliability: number | null;
  sources: Source[];
  notes: string | null;
  /** Official items: the rumor that this move confirmed. */
  rumorId: string | null;
  /** Confirmed rumors: the official move that confirmed it. */
  transferId: string | null;
}

export interface FeedResponse {
  items: FeedItem[];
  nextCursor: string | null;
}

export interface StatsResponse {
  window: StatsWindow;
  since: string;
  totalMoves: number;
  byLeagueAndKind: { league: League; kind: MoveKind; count: number }[];
  freeAgents: number;
  topTeamsByChurn: { team: string; league: string | null; arrivals: number; departures: number }[];
  rumorConfirmationRate: number | null;
}

export interface FlowsResponse {
  level: "league" | "team";
  window: StatsWindow;
  since: string;
  /** Outside the tracked leagues the league level uses "Other". */
  links: { source: string; target: string; value: number }[];
}

export interface TeamInfo {
  name: string;
  short: string | null;
  league: string | null;
  region: string | null;
  logoUrl: string | null;
}

export interface TeamsResponse {
  /** Teams currently in a tracked league, by name. */
  teams: { name: string; short: string | null; league: League }[];
}

export interface RosterEntry {
  player: string;
  role: string | null;
}

export interface Rumor {
  id: string;
  status: RumorStatus;
  player: string;
  fromTeam: string | null;
  toTeam: string | null;
  role: string | null;
  kind: string | null;
  reportedAt: string;
  reliability: number;
  sources: Source[];
  notes: string | null;
  transferId: string | null;
}

export interface TeamResponse {
  team: TeamInfo;
  /** November 1 that started the current season. */
  seasonStart: string;
  /** Last tournament before `seasonStart`, the source of `previousRoster`. */
  previousTournament: string | null;
  roster: RosterEntry[];
  previousRoster: RosterEntry[];
  arrivals: string[];
  departures: string[];
  rumors: Rumor[];
}

export interface PlayerResponse {
  player: {
    name: string;
    realName: string | null;
    country: string | null;
    role: string | null;
    imageUrl: string | null;
  };
  currentTeam: string | null;
  currentTeamShort: string | null;
  timeline: FeedItem[];
  rumors: FeedItem[];
}
