import type { FeedStatus, MoveKind } from "@lol/shared";

export const LEAGUES = ["LCK", "LPL", "LEC", "LCS"] as const;
export const ROLES = ["Top", "Jungle", "Mid", "Bot", "Support"] as const;

export const KINDS = [
  "transfer",
  "free_agent_signing",
  "release",
  "loan",
  "promotion",
  "bench",
  "sub",
  "retirement",
] as const satisfies readonly MoveKind[];

export const STATUSES = [
  "official",
  "confirmed",
  "rumor",
  "denied",
  "stale",
  "contradicted",
] as const satisfies readonly FeedStatus[];

export const KIND_LABELS: Record<MoveKind, string> = {
  transfer: "Transfer",
  free_agent_signing: "Free agent signing",
  release: "Release",
  loan: "Loan",
  promotion: "Promotion",
  bench: "Benched",
  sub: "Substitute",
  retirement: "Retirement",
};

export const STATUS_LABELS: Record<FeedStatus, string> = {
  official: "Official",
  confirmed: "Confirmed",
  rumor: "Rumor",
  denied: "Denied",
  stale: "Faded",
  contradicted: "Contradicted",
};

export function kindLabel(kind: string | null): string | null {
  const known = KINDS.find((item) => item === kind);

  return known ? KIND_LABELS[known] : null;
}

const DAY_MS = 86_400_000;

export function teamCode(name: string, short: string | null): string {
  if (short) return short;

  const words = name.split(/\s+/).filter(Boolean);

  if (words.length === 1) return name.slice(0, 3).toUpperCase();

  return words
    .map((word) => word[0])
    .join("")
    .slice(0, 4)
    .toUpperCase();
}

// Leaguepedia disambiguates shared handles as "Name (Real Name)"; the handle alone reads better.
export function playerHandle(player: string): string {
  return player.replace(/\s+\(.*\)$/, "");
}

export function relativeDay(isoDay: string, now: Date): string {
  const today = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  const days = Math.round((today - Date.parse(isoDay)) / DAY_MS);

  if (days <= 0) return "Today";

  if (days < 30) return `${days}d ago`;

  return shortDate(isoDay, isoDay.slice(0, 4) !== String(now.getUTCFullYear()));
}

export function shortDate(iso: string, withYear = false): string {
  return new Date(iso).toLocaleDateString("en", {
    month: "short",
    day: "numeric",
    year: withYear ? "numeric" : undefined,
    timeZone: "UTC",
  });
}

export function localToday(): string {
  const date = new Date();

  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

export function minutesAgo(iso: string, now: Date): number {
  return Math.max(0, Math.floor((now.getTime() - Date.parse(iso)) / 60_000));
}

export function formatAge(minutes: number): string {
  if (minutes < 1) return "just now";

  if (minutes < 60) return `${minutes} min ago`;

  const hours = Math.floor(minutes / 60);

  return hours === 1 ? "1 hour ago" : `${hours} hours ago`;
}

export function tournamentLabel(page: string): string {
  return page
    .split("/")
    .map((part) => part.replace(/\s+Season$/, ""))
    .join(" ");
}
