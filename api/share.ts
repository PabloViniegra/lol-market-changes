import { NotFound, text } from "./_lib/http.js";
import { loadPlayer } from "./_lib/player.js";
import { playerSubject, shareHtml, teamSubject, type ShareSubject } from "./_lib/share.js";
import { loadTeam } from "./_lib/team.js";

async function subject(params: URLSearchParams): Promise<ShareSubject | null> {
  const team = text(params, "team");
  const player = text(params, "player");

  if (team) return teamSubject(await loadTeam(team));

  if (player) return playerSubject(await loadPlayer(player));

  return null;
}

// A failed lookup still answers with the site-wide card so previews never break.
export async function GET(request: Request): Promise<Response> {
  const url = new URL(request.url);
  let found: ShareSubject | null = null;

  try {
    found = await subject(url.searchParams);
  } catch (error) {
    if (!(error instanceof NotFound)) console.error(error);
  }

  return new Response(shareHtml(found, url.origin), {
    headers: {
      "Content-Type": "text/html; charset=utf-8",
      "Cache-Control": "public, s-maxage=300, stale-while-revalidate=600",
    },
  });
}
