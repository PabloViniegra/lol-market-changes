import type { FeedItem, PlayerResponse, TeamResponse } from "@lol/shared";

export const SITE_NAME = "Transfer Board";
export const DEFAULT_DESCRIPTION =
  "Follow League of Legends esports transfers, roster changes and rumors across the LCK, LPL, LEC and LCS.";

export interface ShareSubject {
  path: string;
  query: string;
  title: string;
  description: string;
}

export function playerHandle(player: string): string {
  return player.replace(/\s+\(.*\)$/, "");
}

export function shortDate(iso: string): string {
  return new Date(iso).toLocaleDateString("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  });
}

export function latestMove(data: PlayerResponse): FeedItem | null {
  return data.timeline[0] ?? null;
}

export function teamSubject(data: TeamResponse): ShareSubject {
  const { name } = data.team;
  const roster = data.roster.map((entry) => playerHandle(entry.player));
  const since = shortDate(data.seasonStart);
  const changes = `${data.arrivals.length} in and ${data.departures.length} out since ${since}.`;

  return {
    path: `/teams/${encodeURIComponent(name)}`,
    query: `team=${encodeURIComponent(name)}`,
    title: `${name} roster`,
    description: roster.length ? `Current roster: ${roster.join(", ")}. ${changes}` : changes,
  };
}

export function playerSubject(data: PlayerResponse): ShareSubject {
  const { name } = data.player;
  const handle = playerHandle(name);
  const move = latestMove(data);
  const team = data.currentTeam ? `Plays for ${data.currentTeam}.` : "No current team.";
  const moveText = move
    ? ` Latest move: ${move.fromTeam ?? "free agent"} to ${move.toTeam ?? "no team"} on ${shortDate(move.date)}.`
    : "";

  return {
    path: `/players/${encodeURIComponent(name)}`,
    query: `player=${encodeURIComponent(name)}`,
    title: handle,
    description: `${team}${moveText}`,
  };
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

// Only link-preview bots get this page (see vercel.json), so it carries the tags and a link, not the app.
export function shareHtml(subject: ShareSubject | null, origin: string): string {
  const title = subject ? `${subject.title} · ${SITE_NAME}` : SITE_NAME;
  const description = subject?.description ?? DEFAULT_DESCRIPTION;
  const url = new URL(subject?.path ?? "/", origin).href;
  const image = new URL(subject ? `/api/og?${subject.query}` : "/api/og", origin).href;
  const tags: [string, string, string][] = [
    ["name", "description", description],
    ["property", "og:site_name", SITE_NAME],
    ["property", "og:type", "website"],
    ["property", "og:title", title],
    ["property", "og:description", description],
    ["property", "og:url", url],
    ["property", "og:image", image],
    ["property", "og:image:width", "1200"],
    ["property", "og:image:height", "630"],
    ["name", "twitter:card", "summary_large_image"],
  ];
  const meta = tags
    .map(([key, name, content]) => `<meta ${key}="${name}" content="${escapeHtml(content)}" />`)
    .join("\n    ");

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>${escapeHtml(title)}</title>
    <link rel="canonical" href="${escapeHtml(url)}" />
    ${meta}
  </head>
  <body>
    <a href="${escapeHtml(url)}">${escapeHtml(title)}</a>
  </body>
</html>
`;
}
