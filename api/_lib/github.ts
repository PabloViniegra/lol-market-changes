import { appendRumor, hasRumor, rumorId, type Submission } from "./submission.js";

const REPO = "PabloViniegra/lol-market-changes";
const BRANCH = "main";
const FILE = "data/rumors.yaml";
const WORKFLOW = "ingest.yml";

async function github(path: string, init: RequestInit = {}): Promise<Response> {
  const token = process.env.GITHUB_TOKEN;
  if (!token) throw new Error("GITHUB_TOKEN is not configured");
  return fetch(`https://api.github.com/repos/${REPO}${path}`, {
    ...init,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "User-Agent": "transfer-board-rumors",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
}

/** Commits the rumor to `data/rumors.yaml`; the ingestor syncs it into the database. */
export async function addRumor(rumor: Submission): Promise<"added" | "exists"> {
  const id = rumorId(rumor);
  // A concurrent commit makes the stale sha fail with 409, so re-read and retry once.
  for (let attempt = 0; attempt < 2; attempt++) {
    const current = await github(`/contents/${FILE}?ref=${BRANCH}`);
    if (!current.ok) throw new Error(`GitHub read failed with ${current.status}`);
    const { content, sha } = (await current.json()) as { content: string; sha: string };
    const yaml = Buffer.from(content, "base64").toString("utf8");
    if (hasRumor(yaml, id)) return "exists";

    const update = await github(`/contents/${FILE}`, {
      method: "PUT",
      body: JSON.stringify({
        message: `feat(rumors): add ${id}`,
        content: Buffer.from(appendRumor(yaml, rumor)).toString("base64"),
        sha,
        branch: BRANCH,
      }),
    });
    if (update.ok) return "added";
    if (update.status !== 409 || attempt === 1) {
      throw new Error(`GitHub commit failed with ${update.status}`);
    }
  }
  throw new Error("unreachable");
}

/** Runs the ingestor now instead of waiting for the next scheduled run. */
export async function triggerIngest(): Promise<boolean> {
  const response = await github(`/actions/workflows/${WORKFLOW}/dispatches`, {
    method: "POST",
    body: JSON.stringify({ ref: BRANCH }),
  });
  if (!response.ok) console.error(`Ingest dispatch failed with ${response.status}`);
  return response.ok;
}
