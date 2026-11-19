import assert from "node:assert/strict";
import { afterEach, beforeEach, describe, it, mock } from "node:test";
import {
  appendRumor,
  createToken,
  hasRumor,
  isSpam,
  latestReportDate,
  parseSubmission,
  readToken,
  rumorId,
  type Submission,
} from "./submission.js";

const valid = {
  player: "Faker",
  fromTeam: "T1",
  toTeam: "Gen.G",
  role: "Mid",
  kind: "transfer",
  reportedAt: "2026-11-18",
  sourceUrl: "https://example.com/report",
  outlet: "Example News",
  reliability: 4,
  notes: "Line one\nLine two",
};

function accept(body: object): Submission {
  const result = parseSubmission(body);
  assert.ok("rumor" in result, "error" in result ? result.error : "");
  return result.rumor;
}

function reject(body: object): string {
  const result = parseSubmission(body);
  assert.ok("error" in result, "expected the submission to be rejected");
  return result.error;
}

describe("parseSubmission", () => {
  beforeEach(() => mock.timers.enable({ apis: ["Date"], now: Date.parse("2026-11-18T12:00:00Z") }));
  afterEach(() => mock.timers.reset());

  it("accepts a complete rumor", () => {
    assert.deepEqual(accept(valid), valid);
  });

  it("normalises blank optional fields to null and trims text", () => {
    const rumor = accept({
      player: "  Faker ",
      fromTeam: "",
      toTeam: "T1",
      outlet: "   ",
      notes: "",
      reportedAt: "2026-11-18",
      sourceUrl: "https://example.com",
      reliability: 3,
    });
    assert.equal(rumor.player, "Faker");
    assert.equal(rumor.fromTeam, null);
    assert.equal(rumor.outlet, null);
    assert.equal(rumor.notes, null);
    assert.equal(rumor.role, null);
    assert.equal(rumor.kind, null);
  });

  it("requires at least one team", () => {
    assert.match(reject({ ...valid, fromTeam: null, toTeam: "" }), /fromTeam or toTeam/);
  });

  it("accepts a report dated today in Korea while it is still yesterday in UTC", () => {
    mock.timers.setTime(Date.parse("2026-11-18T20:00:00Z"));
    assert.equal(accept({ ...valid, reportedAt: "2026-11-19" }).reportedAt, "2026-11-19");
  });

  it("rejects a date after today everywhere on Earth", () => {
    mock.timers.setTime(Date.parse("2026-11-18T09:59:59Z"));
    assert.match(reject({ ...valid, reportedAt: "2026-11-19" }), /^reportedAt: is in the future/);
  });

  it("rejects malformed dates", () => {
    assert.match(reject({ ...valid, reportedAt: "2026-02-30" }), /^reportedAt:/);
    assert.match(reject({ ...valid, reportedAt: "18/11/2026" }), /^reportedAt:/);
  });

  it("only accepts http and https sources", () => {
    assert.match(reject({ ...valid, sourceUrl: "javascript:alert(1)" }), /^sourceUrl:/);
    assert.match(reject({ ...valid, sourceUrl: "ftp://example.com" }), /^sourceUrl:/);
    assert.equal(
      accept({ ...valid, sourceUrl: "http://example.com" }).sourceUrl,
      "http://example.com",
    );
  });

  it("rejects reliability outside 1 to 5", () => {
    assert.match(reject({ ...valid, reliability: 0 }), /^reliability:/);
    assert.match(reject({ ...valid, reliability: 6 }), /^reliability:/);
    assert.match(reject({ ...valid, reliability: 2.5 }), /^reliability:/);
  });

  it("rejects characters that would break rumors.yaml parsing", () => {
    for (const bad of [
      "Fa\u007Fker",
      "Fa\u0085ker",
      "Fa\uD800ker",
      "Fa\nker",
      "Fa\u2028ker",
      "Fa\uFEFFker",
    ]) {
      assert.match(
        reject({ ...valid, player: bad }),
        /^player: contains hidden/,
        JSON.stringify(bad),
      );
    }
    assert.match(reject({ ...valid, notes: "ok\u0000" }), /^notes: contains hidden/);
  });

  it("keeps accented and non-Latin names and newlines in notes", () => {
    const rumor = accept({ ...valid, player: "Ñoño 페이커", notes: "a\nb" });
    assert.equal(rumor.player, "Ñoño 페이커");
    assert.equal(rumor.notes, "a\nb");
  });
});

describe("latestReportDate", () => {
  it("is the calendar date in UTC+14", () => {
    assert.equal(latestReportDate(Date.parse("2026-11-18T09:59:59Z")), "2026-11-18");
    assert.equal(latestReportDate(Date.parse("2026-11-18T10:00:00Z")), "2026-11-19");
  });
});

describe("isSpam", () => {
  it("flags submissions that fill the hidden website field", () => {
    assert.equal(isSpam({ ...valid, website: "http://spam.example" }), true);
    assert.equal(isSpam({ ...valid, website: "" }), false);
    assert.equal(isSpam(null), false);
  });
});

describe("review tokens", () => {
  const now = Date.parse("2026-11-18T12:00:00Z");
  const day = 86_400_000;
  let rumor: Submission;

  beforeEach(() => {
    process.env.REVIEW_SECRET = "s".repeat(32);
    mock.timers.enable({ apis: ["Date"], now });
    rumor = accept(valid);
  });
  afterEach(() => mock.timers.reset());

  it("round-trips the submission", () => {
    assert.deepEqual(readToken(createToken(rumor, now), now), rumor);
  });

  it("expires after 14 days", () => {
    const token = createToken(rumor, now);
    assert.deepEqual(readToken(token, now + 14 * day), rumor);
    assert.equal(readToken(token, now + 14 * day + 1), null);
  });

  it("rejects a token whose payload was changed", () => {
    const [, signature] = createToken(rumor, now).split(".");
    const forged = Buffer.from(
      JSON.stringify({ s: { ...rumor, toTeam: "Other" }, exp: now + day }),
    ).toString("base64url");
    assert.equal(readToken(`${forged}.${signature}`, now), null);
  });

  it("rejects a token signed with another secret", () => {
    const token = createToken(rumor, now);
    process.env.REVIEW_SECRET = "t".repeat(32);
    assert.equal(readToken(token, now), null);
  });

  it("rejects malformed tokens", () => {
    for (const token of ["", "abc", "a.b.c", ".", `${createToken(rumor, now)}.x`]) {
      assert.equal(readToken(token, now), null, token);
    }
  });

  it("refuses to sign without a strong secret", () => {
    process.env.REVIEW_SECRET = "short";
    assert.throws(() => createToken(rumor, now), /REVIEW_SECRET/);
  });
});

describe("rumors.yaml entries", () => {
  beforeEach(() => mock.timers.enable({ apis: ["Date"], now: Date.parse("2026-11-18T12:00:00Z") }));
  afterEach(() => mock.timers.reset());

  it("builds a readable id from the date, player and teams", () => {
    assert.equal(rumorId(accept(valid)), "2026-11-18-faker-t1-to-gen-g");
    assert.equal(
      rumorId(accept({ ...valid, player: "Rekkles Ñ", fromTeam: null, toTeam: "Fnatic" })),
      "2026-11-18-rekkles-n-free-agent-to-fnatic",
    );
    assert.equal(rumorId(accept({ ...valid, toTeam: null })), "2026-11-18-faker-t1-to-out");
  });

  it("replaces an empty list with the first rumor", () => {
    const yaml = appendRumor("# Rumors\n[]\n", accept(valid));
    assert.equal(
      yaml,
      [
        "# Rumors",
        '- id: "2026-11-18-faker-t1-to-gen-g"',
        '  player: "Faker"',
        '  from_team: "T1"',
        '  to_team: "Gen.G"',
        "  role: Mid",
        "  kind: transfer",
        "  reported_at: 2026-11-18",
        "  sources:",
        '    - url: "https://example.com/report"',
        '      outlet: "Example News"',
        "  reliability: 4",
        "  status: rumor",
        '  notes: "Line one\\nLine two"',
        "",
      ].join("\n"),
    );
  });

  it("appends after existing rumors and leaves out unset optional fields", () => {
    const first = appendRumor("[]\n", accept(valid));
    const second = accept({
      ...valid,
      player: "Caps",
      role: null,
      kind: null,
      outlet: null,
      notes: null,
    });
    const yaml = appendRumor(first, second);
    assert.ok(yaml.startsWith(first.trimEnd() + "\n\n- id:"));
    const added = yaml.slice(first.length);
    assert.doesNotMatch(added, /role:|kind:|outlet:|notes:/);
    assert.ok(hasRumor(yaml, rumorId(second)));
  });

  it("quotes user text so YAML syntax in it stays plain text", () => {
    const rumor = accept({
      ...valid,
      player: 'x: "y" # z',
      outlet: "- [a]",
      notes: "&anchor *ref",
    });
    const yaml = appendRumor("[]\n", rumor);
    assert.match(yaml, /^ {2}player: "x: \\"y\\" # z"$/m);
    assert.match(yaml, /^ {6}outlet: "- \[a\]"$/m);
    assert.match(yaml, /^ {2}notes: "&anchor \*ref"$/m);
  });

  it("finds rumors by id whether or not the id is quoted", () => {
    const yaml = '- id: 2026-01-01-a-b-to-c\n  player: A\n\n- id: "2026-01-02-d-e-to-f"\n';
    assert.ok(hasRumor(yaml, "2026-01-01-a-b-to-c"));
    assert.ok(hasRumor(yaml, "2026-01-02-d-e-to-f"));
    assert.ok(!hasRumor(yaml, "2026-01-01-a-b-to"));
  });
});
