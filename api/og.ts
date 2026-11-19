import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { Resvg, initWasm } from "@resvg/resvg-wasm";
import satori from "satori";
import { BODY, DISPLAY, defaultCard, playerCard, teamCard, type CardNode } from "./_lib/og.js";
import { BadRequest, NotFound, text } from "./_lib/http.js";
import { loadPlayer } from "./_lib/player.js";
import { loadTeam } from "./_lib/team.js";

const WIDTH = 1200;
const HEIGHT = 630;

const font = (file: string) => readFile(new URL(`./_fonts/${file}`, import.meta.url));

const ready = Promise.all([
  font("big-shoulders-display-latin-800-normal.woff"),
  font("atkinson-hyperlegible-next-latin-400-normal.woff"),
  font("atkinson-hyperlegible-next-latin-700-normal.woff"),
  readFile(createRequire(import.meta.url).resolve("@resvg/resvg-wasm/index_bg.wasm")).then(
    initWasm,
  ),
]);

async function card(params: URLSearchParams): Promise<CardNode> {
  const team = text(params, "team");
  const player = text(params, "player");

  try {
    if (team) return teamCard(await loadTeam(team));

    if (player) return playerCard(await loadPlayer(player));
  } catch (error) {
    if (!(error instanceof NotFound || error instanceof BadRequest)) throw error;
  }

  return defaultCard();
}

export async function GET(request: Request): Promise<Response> {
  try {
    const [[display, body, bodyBold], element] = await Promise.all([
      ready,
      card(new URL(request.url).searchParams),
    ]);
    const svg = await satori(element, {
      width: WIDTH,
      height: HEIGHT,
      fonts: [
        { name: DISPLAY, data: display, weight: 800, style: "normal" },
        { name: BODY, data: body, weight: 400, style: "normal" },
        { name: BODY, data: bodyBold, weight: 700, style: "normal" },
      ],
    });
    const png = new Resvg(svg, { fitTo: { mode: "width", value: WIDTH } }).render().asPng();

    return new Response(new Uint8Array(png), {
      headers: {
        "Content-Type": "image/png",
        "Cache-Control": "public, s-maxage=3600, stale-while-revalidate=86400",
      },
    });
  } catch (error) {
    console.error(error);

    return new Response("Internal error", { status: 500 });
  }
}
