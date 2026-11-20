# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Ingestor: rumors with a destination become `contradicted` when, within the match window, the player officially moves to a third team instead (not back to the rumored origin), so most dead rumors close without editing `rumors.yaml`. A later match still confirms them and a manual `denied` still wins. Migration `0007` allows the new status.
- API and web: the `contradicted` status in the feed filter, the rumor tracker's closed list, team and compare rumors, the board legend and the rumor confirmation rate, with a muted dashed `Contradicted` pill.
- Web: roster comparison (`/compare?a=…&b=…`, in the header and linked from each team page) shows the current rosters of two teams side by side by role, with team logos, players who joined and left since the season start, rumors linking the two teams and a swap button. An unknown team is flagged on its field with a "Did you mean" suggestion while the other team still shows, an empty field is pointed out on submit, and so is picking the same team twice. Team and player names use the board's large type, and picking or swapping a team flips its header flap.
- API: `GET /api/og` draws a 1200×630 split-flap link preview card for a team (`?team=`, roster and season changes), a player (`?player=`, current team and latest move) or the site, with satori and resvg-wasm. Team and player pages answer link-preview bots (X, Discord, Slack, WhatsApp and others, matched by user agent in `vercel.json`) with `GET /api/share`, a page with their title, description and card; visitors and search engines still get the app. The other pages share the site card.
- Web: keyboard shortcuts: `j`/`k` move between board rows, `g` then `d`, `f`, `r` or `c` opens a screen, and `?` (or "Keyboard shortcuts" in the footer) lists them.
- Web: screen changes turn the page content like a large split-flap around its seam, the nav underline slides to the new tab and the footer follows the new height (a short fade with reduced motion). Filter changes on the same screen don't turn.
- Web: privacy and data credits pages, a privacy notice at rumor submission, and the software and font license notices in the published site. The privacy page opens with a board of where your data goes and a contact panel with a copy-email button, and its section links can be shared; credits list the licenses as board rows.
- Web: "Send a rumor" form (`/rumors/new`, linked from the rumor tracker) validated with vee-validate and Zod, grouped into the move and the source, with team suggestions, a live preview of the board row on wide screens, a specific message when the rate limit is hit and a spam honeypot.
- API: `POST /api/submissions` emails each submitted rumor through Resend, in the Transfer Board design, with signed accept and reject links. `/api/review` asks for confirmation and, on accept, commits the rumor to `data/rumors.yaml` and starts the Ingest workflow. Needs `RESEND_API_KEY`, `REVIEW_EMAIL_TO`, `REVIEW_SECRET` and `GITHUB_TOKEN` in Vercel.
- Security: a Vercel WAF rule limits `POST /api/submissions` to 3 requests per IP every 10 minutes, and submissions with control or hidden characters are rejected so an accepted rumor can't break `rumors.yaml` parsing.
- API: unit tests for rumor submissions (validation, review tokens and `rumors.yaml` entries) with `node:test` and tsx; `pnpm test` runs them and is part of `pnpm check` and CI.
- Web: League of Legends wordmark at the right end of the header and a Riot Games non-endorsement disclaimer in the footer.
- Web: team logos on flaps and a team crest, player photos on board rows, rosters and the player page, and minimap-style role icons. Near-black logos sit on a light plate so they stay visible on petrol.
- Ingestor: store every player on a tracked tournament roster, not only players with moves, and fetch each player's photo from the latest tournament in the last two years (`PlayerImages`) when Leaguepedia has no profile image.
- API: feed items include `fromLogo`, `toLogo` and `playerImage`; roster entries include `imageUrl`; the player response includes `currentTeamLogo`. Images are served as Leaguepedia CDN thumbnails.
- Web: dashboard (`/`) with the latest moves board, per-league move counts, free agents, rumor confirmation rate, busiest teams, a league or team flows Sankey and rumors to watch.
- Web: market feed (`/feed`) with league, role, move type, status and date filters mirrored in the URL, and paginated loading.
- Web: team page (`/teams/:name`) with the previous and current roster, arrivals, departures, moves since the season start and related rumors.
- Web: "Updated X min ago" indicator, a stale-data banner after one hour without a sync, error states that retry every 30 seconds and a one-time flap flip on new moves (a yellow edge with reduced motion). Data refetches every 5 minutes.
- Web: Pinia dependency; the dev server proxies `/api` to production, or to `API_ORIGIN` when set.
- Web: static loading skeletons that mirror the board rows, dashboard stats, flows chart, team rosters and player header.
- API: feed items include `fromShort` and `toShort` team codes.
- Web: Transfer Board design system (`DESIGN.md`) as Tailwind CSS 4 tokens and `type-*` utilities, with self-hosted Big Shoulders Display and Atkinson Hyperlegible Next fonts; `PRODUCT.md` with the product context.
- Web: split-flap arrow logo in the header, WebP and ICO favicons, an Apple touch icon and the theme colour.
- MIT license.
- This changelog.
- Local Graphify source graph configuration and Git hook setup instructions.
- Ingestor: Leaguepedia Cargo client with pagination (500 rows per request), a 6 s request interval and exponential backoff on `ratelimited`, timeouts, connection errors and 5xx responses.
- Ingestor: `probe` binary that dumps one month of roster changes, tournaments, teams, players, redirects and tenures to JSON fixtures.
- Ingestor: database migrations for `teams`, `team_seasons`, `players`, `raw_events` and `sync_state`.
- Ingestor: fetch and normalisation of LCK, LPL, LEC and LCS roster changes (LTA North counts as LCS), with team and player redirects resolved, staff and Game Changers rows skipped, and the full history of in-scope players kept.
- Ingestor: idempotent upserts into Neon, soft deletion of events removed from the wiki, incremental runs with a 14-day overlap, `--full` two-season backfill and `--since`; each run is recorded in `sync_state`.
- Ingestor: pairing of roster changes into transfers (transfer, free-agent signing, release, retirement, loan and loan return, promotion, bench and sub), with a 45-day window, renamed teams treated as one team and contract extensions or role swaps ignored. Each run rebuilds the transfers of the players it touched.
- Ingestor: `transfers` table, `teams.renamed_to` column and `leaguepedia_cache` table (migration `0002`).
- Ingestor: incremental runs reuse the tournaments, rosters and scope team redirects fetched in the last 24 hours; `--full` always refreshes them.
- Ingestor: `data/rumors.yaml` parsing and validation (unique ids, a team, past report date, reliability 1–5, at least one source, manual status `rumor` or `denied`), synced to a new `rumors` table (migration `0003`) on every run.
- Ingestor: rumor reconciliation: confirmed by an official move of the same player to the same team (or from it when there is no destination) from 7 days before to 90 days after the report, stale after 60 days without a match, and `denied` never overridden.
- Ingestor: `--rumors <path>` flag, defaulting to `../data/rumors.yaml`.
- CI: `data/rumors.yaml` is validated by the ingestor tests.
- CI: Postgres service for the ingestor database test.
- Ingest workflow runs every 15 minutes (`7,22,37,52 * * * *`) and reuses the cached release binary until the ingestor sources, migrations or toolchain change.
- Ingestor: tournament rosters (in-game players and roles) are stored in `team_seasons.roster` (migration `0004`).
- `market_feed` view with official moves where either team was in a tracked league on the move date, plus every rumor, and a `league_at(team, date)` function (migration `0004`).
- API: `GET /api/feed` with league, team, role, kind, status and date filters and cursor pagination; denied and stale rumors are opt-in.
- API: `GET /api/stats` and `GET /api/flows` over a `7d`, `30d` or `season` window (the season starts on November 1).
- API: `GET /api/teams/:name` with the roster before the season, the current roster, arrivals, departures and related rumors.
- API: `GET /api/players/:name` with the player's current team (from the latest move or, when newer, the latest tournament roster), moves and rumors.
- Fixtures from Leaguepedia in `ingestor/tests/fixtures/probe/` (CC BY-SA).
- Web: anti-slop rules for Oxlint.
- Web: player page (`/players/:name`) with the current team, moves and rumors; player names on boards and team rosters link to it.
- Web: rumor tracker (`/rumors`) with open, confirmed and denied or faded rumors and the confirmation rate, linked from the header and the dashboard.
- Web: "How to read the board" legend explaining team codes, `FA`, `OUT` and every status.
- Web: feed team filter, a visible label for each filter group and a "Showing N moves" count.
- API: `GET /api/teams` lists the teams in a tracked league; the feed team filter suggests them.
- API and web: feed player filter (`?player=`, case-insensitive partial match), always visible in the filters and focused with `/`.
- Web: dashboard "Transfers" and "All moves" toggle for the latest moves, mirrored in the URL (`?moves=all`); transfers are the default.
- Web: feed skip link to "Load more moves" for keyboard users.

### Changed

- Web: Compare distinguishes current players from departures, opens the full team catalog, and applies edits and swaps when you select Compare. On phones, a valid comparison collapses the picker so the rosters appear sooner.
- Web: the paginated market feed refreshes after a successful ingest is detected, with a 15-minute fallback, and keeps the pages already loaded. Feed and flows requests are cancelled when their filters are no longer observed, and the team catalog stays in cache for an hour after leaving a view.
- Web: dropdowns, team suggestions, calendar, reliability dots, live rumor preview and flow routes update immediately. Split-flap motion marks new moves and rumor submission confirmations, and the rumor submit button gives brief press feedback.
- Web: the dashboard "See all" button sits under the latest moves board and the legend at the right of its heading; player names line up across board rows even when a team code is long.
- API: encrypt new review links, keep private notes out of published rumors, and let reviewers clean up the public source URL before approving. Review pages and emails use local system fonts.
- Web: the feed date filters and rumor report date use a shared calendar styled like the dropdowns, with keyboard navigation and the relevant date limits.
- Web: "Send a rumor" form uses its own role and move dropdowns, a team combobox with codes and league tags, and a reliability dot meter instead of native selects and datalists, so every option follows the design system. The player name is typed in board type, the preview flips its flaps when a team changes and the success state shows the rumor waiting for review.
- Web: the feed team filter uses the same team combobox, with codes and league tags.
- Web: "Where players went" is a hand-built SVG flows diagram, even with only a few routes. Pointing at or tabbing through a route (arrow keys move between routes) highlights it and shows its count, team nodes take their league colour, and switching between league and team view morphs the diagram (instant with reduced motion). It replaces the ECharts Sankey and the short-list fallback.
- Web: board rows fit on two lines below 768px: the move type stays available to screen readers only and the team-name subline is hidden, since the flaps carry it.
- Web: an empty rumor tracker explains the rumor lifecycle; the dashboard hides the rumor strip and the confirmation rate when there are no rumors.
- Web: the feed empty state suggests removing only the filters that are set; the dashboard free agent row reads "Released and unsigned".
- Web: moved the full-feed link next to the dashboard heading, clarified the season summary, aligned dates and sources on the board, and made the mobile filter actions stay in view while scrolling.
- Web: shortened the split-flap animation for new moves and added a transition to the mobile filter sheet. Reduced-motion settings remove rotation and sliding.

- Web: consolidated the dashboard summary and clarified how releases and resolved rumors are counted. The rumor tracker uses one empty message, player history foregrounds team destinations, and team rosters compare players by role.

- Ingestor: Leaguepedia queries are sent as POST, so `IN` lists are split into chunks of 20,000 characters instead of 2,000 and runs need fewer requests.
- Ingest workflow timeout raised from 15 to 30 minutes so a full backfill fits.
- Ingest workflow schedule moved to minutes 4, 19, 34 and 49 (still every 15 minutes) so GitHub registers it again.
- Web: the page title is now "Transfer Board".
- Web: `Official` status pills are neutral (solid outline) and yellow fill is kept for `Confirmed`, so changes stand out on a board of official moves; the per-row "Source" link is an external-link icon named after the outlet.
- Web: the stale-data banner has a warning icon and replaces the "Updated" text instead of repeating it.
- Web: the flows Sankey labels nodes with their move count and direction, sizes itself to its nodes and width, and explains which moves it leaves out.
- Web: filter chips, busiest-team links and header links are larger tap targets on small screens.
- Web: "This season" is now "This window", and previous tournaments show readable names instead of the Leaguepedia page path.
- Web: the empty board state spans the full width.
- API: the feed `team` filter and `GET /api/teams/:name`, including its rumors, ignore letter case, so `fnatic` finds Fnatic.
- API: `GET /api/players/:name` returns moves and rumors as feed items, with team codes, leagues and sources, plus `currentTeamShort`; moves outside the tracked leagues are left out, as in the feed.
- Web: an empty feed filtered by team names the team, and the "Moves involving" link only appears for a known team.
- Web: board rows show the full team names under the player, and dates from another year include the year.
- Web: the feed shows League and Status filters up front, with Role, Move, Team and dates behind a toggle; active filters appear as removable chips. On small screens all filters open in a bottom sheet so moves fill the first screen.
- Web: the feed team filter accepts team codes such as `FNC` and says when a team isn't tracked.
- Web: releases and retirements are dimmed on the board so transfers stand out; the date and source link stay together on small screens.
- Web: the flows section lists moves as "from ▸ to" when there are fewer than 8, and Sankey labels wrap instead of being cut off.
- Web: the rumor tracker merges empty sections into one note on how rumors are tracked.
- Web: the dashboard puts open rumors and a tracker link above the latest board, links the season counts to league-filtered feeds, and shares the flow view in the URL. Keyboard links jump directly to the season summary and flows.
- Web: rumor rows name their sources, all board source links have larger mobile tap targets, and the flow list labels moves within a league explicitly.

- Ingestor, API and web: a rumor's report date may be today anywhere on Earth (UTC+14), so a report from Korea or China dated today is no longer rejected as "in the future" before UTC midnight. The form defaults to the user's local date.

### Removed

- Ingestor: the unused `teams.logo_url` column (migration `0006`); the API builds logos from the team name.

### Fixed

- Web: Compare rejects duplicate teams, distinguishes unavailable rosters from empty roles, supports cancelling partial edits and preserves keyboard focus when correcting a name or returning to the picker.
- Web: refresh the market feed on its first successful ingest and after pagination finishes when the two overlap.
- Web and API: validate rumor sources in Safari 17, preserve the moderator-approved URL including Unicode and fragments, and let reviewers replace credential-bearing sources from previously issued review links.
- Web: keep the mobile dashboard layout steady while fonts and sync status load. Fetch team metadata only when the flows chart needs it, and offer a list of full team routes on narrow screens.
- Web: improve form field contrast and required-field labels, add a skip link, announce empty feed results, use Ctrl+/ for player search and wrap long team filters.
- Web: serve crawlable robots and sitemap files, add search descriptions and canonical links for the main pages, restrict SPA rewrites to known routes and cache fingerprinted assets.

- Web: on mobile the "Updated X min ago" text moves under the navigation instead of being cut off at the right edge of the header.
- Web: official moves that confirm a rumor show the Confirmed badge and match the Confirmed filter. The feed shows one row per match when both entries qualify, and the tracker keeps the original report.
- Ingestor and API: record and expose the last successful sync separately from the latest attempt. The dashboard shows failed or unavailable freshness and keeps the Leaguepedia credit visible when metadata is unavailable.
- Web: board empty states and query errors avoid promising a refresh time they cannot guarantee.
- Web: feed date filters can no longer set a start date after the end date.
- Web: inactive filter chips in the desktop feed filter panel are visible again; they used the same colour as the panel.
- Web: the player page shows the same team codes as the rest of the board (`FNC`, not `FNA`), with league tags and source links.
- Web: the current page is underlined in the header navigation again.

## [0.1.0] - 2026-11-18

### Added

- Initial project setup: pnpm workspace with the Vue 3 SPA, Vercel read-only API, shared types and the Rust ingestor skeleton.
- CI workflow with lint, format, typecheck, build and Rust checks; manual ingestion workflow.
- Neon Postgres database with separate read-only and read/write roles.

[Unreleased]: https://github.com/PabloViniegra/lol-market-changes/compare/056d672...HEAD
[0.1.0]: https://github.com/PabloViniegra/lol-market-changes/commit/056d672
