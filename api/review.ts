import { addRumor, triggerIngest } from "./_lib/github.js";
import { readToken } from "./_lib/submission.js";
import { messagePage, reviewPage } from "./_lib/templates.js";

function html(body: string, status = 200): Response {
  return new Response(body, {
    status,
    headers: {
      "Content-Type": "text/html; charset=utf-8",
      "Cache-Control": "no-store",
      "Referrer-Policy": "no-referrer",
      "X-Robots-Tag": "noindex",
      "Content-Security-Policy":
        "default-src 'none'; style-src 'unsafe-inline' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; form-action 'self'; frame-ancestors 'none'",
    },
  });
}

const invalid = () =>
  html(messagePage("Link not valid", "This review link is broken or has expired."), 400);

const trackerLink = (origin: string) => ({
  href: `${origin}/rumors`,
  text: "Open the rumor tracker",
});

// GET only renders a confirmation page: mail scanners open links, so they must not change data.
export async function GET(request: Request): Promise<Response> {
  const params = new URL(request.url).searchParams;
  const token = params.get("token") ?? "";
  const rumor = readToken(token);
  if (!rumor) return invalid();
  return html(reviewPage(rumor, token, params.get("decision") === "reject" ? "reject" : "accept"));
}

export async function POST(request: Request): Promise<Response> {
  const form = await request.formData().catch(() => null);
  const token = form?.get("token");
  const rumor = typeof token === "string" ? readToken(token) : null;
  if (!rumor) return invalid();
  if (form?.get("decision") !== "accept") {
    return html(messagePage("Rumor rejected", "Nothing was saved."));
  }

  const origin = new URL(request.url).origin;
  try {
    if ((await addRumor(rumor)) === "exists") {
      return html(
        messagePage(
          "Already on the board",
          "This rumor is already in rumors.yaml.",
          trackerLink(origin),
        ),
      );
    }
    const syncing = await triggerIngest();
    return html(
      messagePage(
        "Rumor accepted",
        syncing
          ? "It was added to rumors.yaml and a sync started. It shows on the board in a few minutes."
          : "It was added to rumors.yaml. It shows on the board after the next scheduled sync.",
        trackerLink(origin),
      ),
    );
  } catch (error) {
    console.error(error);
    return html(
      messagePage("Couldn’t save the rumor", "GitHub didn’t accept the change. Try again."),
      502,
    );
  }
}
