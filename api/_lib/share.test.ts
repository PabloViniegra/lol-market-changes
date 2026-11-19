import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { FeedItem, PlayerResponse, TeamResponse } from "@lol/shared";
import { DEFAULT_DESCRIPTION, playerSubject, shareHtml, teamSubject } from "./share.js";

const team: TeamResponse = {
  team: { name: "Gen.G", short: "GEN", league: "LCK", region: "Korea", logoUrl: null },
  seasonStart: "2026-11-01",
  previousTournament: null,
  roster: [
    { player: "Chovy", role: "Mid", imageUrl: null },
    { player: "Duro (Ju Min-gyu)", role: "Support", imageUrl: null },
  ],
  previousRoster: [],
  arrivals: ["Duro (Ju Min-gyu)"],
  departures: ["Kiin", "Canyon"],
  rumors: [],
};

const move = {
  status: "official",
  fromTeam: "Fnatic",
  toTeam: null,
  date: "2026-11-19",
} as FeedItem;

const player: PlayerResponse = {
  player: { name: "Lospa", realName: null, country: null, role: "Support", imageUrl: null },
  currentTeam: null,
  currentTeamShort: null,
  currentTeamLogo: null,
  timeline: [move],
  rumors: [],
};

describe("share subjects", () => {
  it("summarises a team roster with handles and season changes", () => {
    const subject = teamSubject(team);

    assert.equal(subject.path, "/teams/Gen.G");
    assert.equal(subject.query, "team=Gen.G");
    assert.equal(
      subject.description,
      "Current roster: Chovy, Duro. 1 in and 2 out since Nov 1, 2026.",
    );
  });

  it("describes a player's latest move, including leaving the scene", () => {
    assert.equal(
      playerSubject(player).description,
      "No current team. Latest move: Fnatic to no team on Nov 19, 2026.",
    );
  });
});

describe("shareHtml", () => {
  it("points the card and canonical URL at the subject", () => {
    const html = shareHtml(teamSubject(team), "https://example.test");

    assert.match(
      html,
      /<meta property="og:image" content="https:\/\/example.test\/api\/og\?team=Gen.G" \/>/,
    );
    assert.match(html, /<link rel="canonical" href="https:\/\/example.test\/teams\/Gen.G" \/>/);
  });

  it("escapes names taken from the wiki", () => {
    const html = shareHtml(
      { ...teamSubject(team), title: `"><script>x</script>` },
      "https://example.test",
    );

    assert.doesNotMatch(html, /<script>/);
    assert.match(html, /&quot;&gt;&lt;script&gt;/);
  });

  it("falls back to the site card when nothing was found", () => {
    const html = shareHtml(null, "https://example.test");

    assert.match(html, /content="https:\/\/example.test\/api\/og"/);
    assert.ok(html.includes(DEFAULT_DESCRIPTION));
  });
});
