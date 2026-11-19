import {
  createCipheriv,
  createDecipheriv,
  createHash,
  createHmac,
  randomBytes,
  timingSafeEqual,
} from "node:crypto";
import type { MoveKind, Role, RumorSubmission } from "@lol/shared";
import { z } from "zod";
import { isIsoDate } from "./http.js";

const ROLES = ["Top", "Jungle", "Mid", "Bot", "Support"] as const satisfies readonly Role[];
const KINDS = [
  "transfer",
  "free_agent_signing",
  "release",
  "loan",
  "promotion",
  "bench",
  "sub",
  "retirement",
] as const satisfies readonly MoveKind[];

const TOKEN_TTL_MS = 14 * 86_400_000;

// Reports are dated where they happen, and Korea or China are ahead of UTC, so "today" is
// today in the earliest timezone (UTC+14). The ingestor applies the same rule.
export const latestReportDate = (now = Date.now()) =>
  new Date(now + 14 * 3_600_000).toISOString().slice(0, 10);

// libyaml rejects these even inside JSON-escaped scalars, and one bad rumor stops every ingest run.
const UNSAFE = /[\p{Cc}\u2028\u2029\uFEFF]/u;
const safe = (multiline: boolean) => (value: string) =>
  value.isWellFormed() && !UNSAFE.test(multiline ? value.replaceAll("\n", "") : value);
const HIDDEN = "contains hidden or control characters";

const optionalText = (max: number, multiline = false) =>
  z
    .string()
    .trim()
    .max(max)
    .refine(safe(multiline), HIDDEN)
    .nullish()
    .transform((value) => value || null);

const legacySchema = z
  .object({
    player: z.string().trim().min(1).max(60).refine(safe(false), HIDDEN),
    fromTeam: optionalText(80),
    toTeam: optionalText(80),
    role: z.enum(ROLES).nullish().default(null),
    kind: z.enum(KINDS).nullish().default(null),
    reportedAt: z
      .string()
      .refine(isIsoDate, "must be a YYYY-MM-DD date")
      .refine((date) => date <= latestReportDate(), "is in the future"),
    sourceUrl: z
      .url({ protocol: /^https?$/ })
      .max(500)
      .refine(safe(false), HIDDEN),
    outlet: optionalText(80),
    reliability: z.number().int().min(1).max(5),
    notes: optionalText(500, true),
  })
  .refine((rumor) => rumor.fromTeam || rumor.toTeam, "fromTeam or toTeam is required");

function isPublicSource(value: string): boolean {
  const url = URL.parse(value);
  return url !== null && !url.username && !url.password;
}

const schema = legacySchema.refine((rumor) => isPublicSource(rumor.sourceUrl), {
  path: ["sourceUrl"],
  message: "must be a public link without credentials",
});

export type Submission = z.output<typeof schema>;

export function parseSubmission(body: unknown): { rumor: Submission } | { error: string } {
  const result = schema.safeParse(body);
  if (result.success) return { rumor: result.data };
  const issue = result.error.issues[0];
  return { error: issue ? `${issue.path.join(".") || "body"}: ${issue.message}` : "Invalid rumor" };
}

export function publicationDraft(rumor: Submission): Submission {
  return {
    ...rumor,
    sourceUrl: isPublicSource(rumor.sourceUrl) ? rumor.sourceUrl : "",
    notes: null,
  };
}

export function isSpam(body: unknown): boolean {
  return Boolean((body as Partial<RumorSubmission> | null)?.website);
}

function secret(): string {
  const value = process.env.REVIEW_SECRET;
  if (!value || value.length < 32) throw new Error("REVIEW_SECRET must be at least 32 characters");
  return value;
}

function sign(payload: string): Buffer {
  return createHmac("sha256", secret()).update(payload).digest();
}

export function createToken(submission: Submission, now = Date.now()): string {
  const nonce = randomBytes(12);
  const cipher = createCipheriv("aes-256-gcm", reviewKey(), nonce);
  const payload = JSON.stringify({ s: submission, exp: now + TOKEN_TTL_MS });
  const encrypted = Buffer.concat([cipher.update(payload, "utf8"), cipher.final()]);
  return ["v2", Buffer.concat([nonce, cipher.getAuthTag(), encrypted]).toString("base64url")].join(
    ".",
  );
}

function reviewKey(): Buffer {
  return createHash("sha256").update("transfer-board-review:v2\0").update(secret()).digest();
}

function readLegacyPayload(payload: string, signature: string): string | null {
  const given = Buffer.from(signature, "base64url");
  const expected = sign(payload);
  if (given.length !== expected.length || !timingSafeEqual(given, expected)) return null;
  return Buffer.from(payload, "base64url").toString("utf8");
}

function decryptPayload(payload: string): string | null {
  const bytes = Buffer.from(payload, "base64url");
  if (bytes.length <= 28 || bytes.toString("base64url") !== payload) return null;
  const decipher = createDecipheriv("aes-256-gcm", reviewKey(), bytes.subarray(0, 12));
  decipher.setAuthTag(bytes.subarray(12, 28));
  return Buffer.concat([decipher.update(bytes.subarray(28)), decipher.final()]).toString("utf8");
}

/** Returns the submission, or null when the token is forged, malformed or expired. */
export function readToken(token: string, now = Date.now()): Submission | null {
  if (token.length > 12_000) return null;
  const [first, second, extra] = token.split(".");
  if (!first || !second || extra !== undefined) return null;
  try {
    const payload = first === "v2" ? decryptPayload(second) : readLegacyPayload(first, second);
    if (payload === null) return null;
    const { s, exp } = JSON.parse(payload);
    if (typeof exp !== "number" || exp < now) return null;
    return (first === "v2" ? schema : legacySchema).parse(s);
  } catch {
    return null;
  }
}

function slug(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function rumorId(rumor: Submission): string {
  const from = rumor.fromTeam ? slug(rumor.fromTeam) : "free-agent";
  const to = rumor.toTeam ? slug(rumor.toTeam) : "out";
  return `${rumor.reportedAt}-${slug(rumor.player) || "player"}-${from}-to-${to}`;
}

// JSON strings are valid YAML double-quoted scalars once the schema has rejected control characters.
const quote = (value: string) => JSON.stringify(value);

export function toYamlEntry(rumor: Submission): string {
  const lines = [
    `- id: ${quote(rumorId(rumor))}`,
    `  player: ${quote(rumor.player)}`,
    `  from_team: ${rumor.fromTeam ? quote(rumor.fromTeam) : "null"}`,
    `  to_team: ${rumor.toTeam ? quote(rumor.toTeam) : "null"}`,
  ];
  if (rumor.role) lines.push(`  role: ${rumor.role}`);
  if (rumor.kind) lines.push(`  kind: ${rumor.kind}`);
  lines.push(
    `  reported_at: ${rumor.reportedAt}`,
    "  sources:",
    `    - url: ${quote(rumor.sourceUrl)}`,
  );
  if (rumor.outlet) lines.push(`      outlet: ${quote(rumor.outlet)}`);
  lines.push(`  reliability: ${rumor.reliability}`, "  status: rumor");
  if (rumor.notes) lines.push(`  notes: ${quote(rumor.notes)}`);
  return `${lines.join("\n")}\n`;
}

export function hasRumor(yaml: string, id: string): boolean {
  return yaml.split("\n").some((line) => {
    const match = /^- id:\s*(.+?)\s*$/.exec(line);
    return match?.[1]?.replace(/^["']|["']$/g, "") === id;
  });
}

export function appendRumor(yaml: string, rumor: Submission): string {
  const entry = toYamlEntry(rumor);
  if (/^\[\][ \t]*$/m.test(yaml)) {
    return `${yaml.replace(/^\[\][ \t]*$/m, entry.trimEnd()).trimEnd()}\n`;
  }
  const body = yaml.trimEnd();
  return body ? `${body}\n\n${entry}` : entry;
}
