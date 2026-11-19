import type { PlayerResponse, TeamResponse } from "@lol/shared";
import { latestMove, playerHandle, shortDate } from "./share.js";

// Satori takes React-like element objects, so cards are built without React or JSX.
export interface CardNode {
  type: string;
  key: null;
  props: { style?: Record<string, string | number>; src?: string; children?: CardChild };
}
type CardChild = CardNode | string | (CardNode | string | null)[] | null;

const COLOR = {
  background: "#0e2a2b",
  raised: "#1a4243",
  primary: "#f2c230",
  text: "#eef3ef",
  muted: "#a9bdb8",
};

const LEAGUE_COLOR: Record<string, string> = {
  LCK: "#7fb7e6",
  LPL: "#f07a63",
  LEC: "#b7a3f0",
  LCS: "#6fcf97",
};

export const DISPLAY = "Big Shoulders Display";
export const BODY = "Atkinson Hyperlegible Next";

function el(
  type: string,
  style: Record<string, string | number>,
  children: CardChild = null,
): CardNode {
  return { type, key: null, props: { style: { display: "flex", ...style }, children } };
}

function img(src: string, size: number): CardNode {
  return {
    type: "img",
    key: null,
    props: { src, style: { width: size, height: size, objectFit: "contain" } },
  };
}

function arrow(size: number, color: string): CardNode {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M2 1l7 4-7 4z" fill="${color}"/></svg>`;

  return img(`data:image/svg+xml,${encodeURIComponent(svg)}`, size);
}

function flap(code: string, { width = 180, height = 88, fontSize = 56 } = {}): CardNode {
  return el(
    "div",
    {
      position: "relative",
      width,
      height,
      flexShrink: 0,
      alignItems: "center",
      justifyContent: "center",
      backgroundColor: COLOR.raised,
      borderRadius: 4,
      fontFamily: DISPLAY,
      fontSize,
      color: COLOR.text,
    },
    [
      code,
      el("div", {
        position: "absolute",
        left: 0,
        right: 0,
        top: height / 2,
        height: 2,
        backgroundColor: COLOR.background,
      }),
    ],
  );
}

function leagueTag(league: string | null): CardNode | null {
  const color = league ? LEAGUE_COLOR[league] : undefined;

  if (!league || !color) return null;

  return el(
    "div",
    {
      padding: "4px 14px",
      borderRadius: 4,
      backgroundColor: color,
      color: COLOR.background,
      fontSize: 28,
      fontWeight: 700,
    },
    league,
  );
}

function frame(children: CardChild): CardNode {
  return el(
    "div",
    {
      width: "100%",
      height: "100%",
      flexDirection: "column",
      justifyContent: "space-between",
      padding: 64,
      backgroundColor: COLOR.background,
      color: COLOR.text,
      fontFamily: BODY,
    },
    [
      el("div", { alignItems: "center", gap: 16, fontFamily: DISPLAY, fontSize: 36 }, [
        arrow(32, COLOR.primary),
        "Transfer Board",
      ]),
      ...(Array.isArray(children) ? children : [children]),
    ],
  );
}

function headline(title: string, details: (CardNode | string | null)[]): CardNode {
  return el("div", { flexDirection: "column", gap: 16, minWidth: 0 }, [
    el("div", { fontFamily: DISPLAY, fontSize: 104, lineHeight: 1 }, title),
    el("div", { alignItems: "center", gap: 16, fontSize: 32, color: COLOR.muted }, details),
  ]);
}

const ROLE_ORDER = ["Top", "Jungle", "Mid", "Bot", "Support"];

function teamCode(name: string | null, short: string | null): string | null {
  if (short) return short;

  if (!name) return null;

  const words = name.split(/\s+/).filter(Boolean);

  return words.length === 1
    ? name.slice(0, 3).toUpperCase()
    : words
        .map((word) => word[0])
        .join("")
        .slice(0, 4)
        .toUpperCase();
}

function rolePosition(role: string | null): number {
  const index = role ? ROLE_ORDER.indexOf(role) : -1;

  return index < 0 ? ROLE_ORDER.length : index;
}

function bigFlap(code: string): CardNode {
  return flap(code, { width: 320, height: 200, fontSize: 128 });
}

export function teamCard(data: TeamResponse): CardNode {
  const { team } = data;
  const roster = data.roster
    .toSorted((a, b) => rolePosition(a.role) - rolePosition(b.role))
    .slice(0, 5);

  return frame([
    el("div", { alignItems: "center", gap: 48 }, [
      bigFlap(teamCode(team.name, team.short) ?? "—"),
      headline(team.name, [
        leagueTag(team.league),
        `${data.arrivals.length} in · ${data.departures.length} out since ${shortDate(data.seasonStart)}`,
      ]),
    ]),
    el(
      "div",
      { gap: 12 },
      roster.length
        ? roster.map((entry) =>
            el(
              "div",
              {
                flex: 1,
                flexDirection: "column",
                gap: 4,
                padding: "12px 16px",
                borderRadius: 4,
                backgroundColor: COLOR.raised,
              },
              [
                el("div", { fontSize: 22, color: COLOR.muted }, entry.role ?? "—"),
                el(
                  "div",
                  { fontFamily: DISPLAY, fontSize: 40, color: COLOR.text },
                  playerHandle(entry.player),
                ),
              ],
            ),
          )
        : el("div", { fontSize: 32, color: COLOR.muted }, "No current roster recorded"),
    ),
  ]);
}

export function playerCard(data: PlayerResponse): CardNode {
  const move = latestMove(data);
  const details = [data.player.role, data.currentTeam ?? "No current team"].filter(Boolean);

  return frame([
    el("div", { alignItems: "center", gap: 48 }, [
      bigFlap(teamCode(data.currentTeam, data.currentTeamShort) ?? "FA"),
      headline(playerHandle(data.player.name), [details.join(" · ")]),
    ]),
    move
      ? el("div", { alignItems: "center", gap: 24 }, [
          flap(teamCode(move.fromTeam, move.fromShort) ?? "FA"),
          arrow(40, COLOR.muted),
          flap(teamCode(move.toTeam, move.toShort) ?? "OUT"),
          el("div", { flexDirection: "column", marginLeft: 16, fontSize: 30, color: COLOR.muted }, [
            el("div", { color: COLOR.text }, "Latest move"),
            el("div", {}, shortDate(move.date)),
          ]),
        ])
      : el("div", { fontSize: 32, color: COLOR.muted }, "No moves in the last two seasons"),
  ]);
}

export function defaultCard(): CardNode {
  return frame(
    el("div", { flexDirection: "column", gap: 24 }, [
      el(
        "div",
        { fontFamily: DISPLAY, fontSize: 120, lineHeight: 1 },
        "League of Legends esports transfers",
      ),
      el("div", { alignItems: "center", gap: 16, fontSize: 32, color: COLOR.muted }, [
        el("div", {}, "Official moves and rumors"),
        ...["LCK", "LPL", "LEC", "LCS"].map(leagueTag),
      ]),
    ]),
  );
}
