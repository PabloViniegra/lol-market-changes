# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Web: dashboard (`/`) with the latest moves board, per-league move counts, free agents, rumor confirmation rate, busiest teams, a league or team flows Sankey and rumors to watch.
- Web: market feed (`/feed`) with league, role, move type, status and date filters mirrored in the URL, and paginated loading.
- Web: team page (`/teams/:name`) with the previous and current roster, arrivals, departures, moves since the season start and related rumors.
- Web: "Updated X min ago" indicator, a stale-data banner after one hour without a sync, error states that retry every 30 seconds and a one-time flap flip on new moves (a yellow edge with reduced motion). Data refetches every 5 minutes.
- Web: Pinia, ECharts and vue-echarts dependencies; the dev server proxies `/api` to production, or to `API_ORIGIN` when set.
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

### Changed

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
- Web: the rumor tracker merges empty sections into one note on how rumors are tracked, and the dashboard hides "Rumors to watch" when there are none.

### Fixed

- Web: feed date filters can no longer set a start date after the end date.
- Web: the player page shows the same team codes as the rest of the board (`FNC`, not `FNA`), with league tags and source links.
- Web: the current page is underlined in the header navigation again.

## [0.1.0] - 2026-11-18

### Added

- Initial project setup: pnpm workspace with the Vue 3 SPA, Vercel read-only API, shared types and the Rust ingestor skeleton.
- CI workflow with lint, format, typecheck, build and Rust checks; manual ingestion workflow.
- Neon Postgres database with separate read-only and read/write roles.

[Unreleased]: https://github.com/PabloViniegra/lol-market-changes/compare/056d672...HEAD
[0.1.0]: https://github.com/PabloViniegra/lol-market-changes/commit/056d672
