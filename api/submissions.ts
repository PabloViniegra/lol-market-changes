import type { ErrorResponse, SubmissionResponse } from "@lol/shared";
import { sendEmail } from "./_lib/email.js";
import { createToken, isSpam, parseSubmission } from "./_lib/submission.js";
import { reviewEmail } from "./_lib/templates.js";

function fail(status: number, error: string): Response {
  const body: ErrorResponse = { error };
  return Response.json(body, { status });
}

export async function POST(request: Request): Promise<Response> {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return fail(400, "Body must be JSON");
  }
  const ok: SubmissionResponse = { ok: true };
  // Bots get the same answer as people so they don't learn about the honeypot.
  if (isSpam(body)) return Response.json(ok, { status: 202 });

  const parsed = parseSubmission(body);
  if ("error" in parsed) return fail(400, parsed.error);

  try {
    const token = createToken(parsed.rumor);
    const origin = new URL(request.url).origin;
    await sendEmail(
      reviewEmail(
        parsed.rumor,
        (decision) =>
          `${origin}/api/review?decision=${decision}&token=${encodeURIComponent(token)}`,
      ),
    );
  } catch (error) {
    console.error(error);
    return fail(500, "Couldn’t send the rumor for review");
  }
  return Response.json(ok, { status: 202 });
}
