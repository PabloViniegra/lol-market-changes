# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- MIT license.
- This changelog.
- Ingestor: Leaguepedia Cargo client with pagination (500 rows per request), a 6 s request interval and exponential backoff on `ratelimited`, timeouts, connection errors and 5xx responses.
- Ingestor: `probe` binary that dumps one month of roster changes, tournaments, teams, players, redirects and tenures to JSON fixtures.
- Ingestor: database migrations for `teams`, `team_seasons`, `players`, `raw_events` and `sync_state`.
- Ingestor: fetch and normalisation of LCK, LPL, LEC and LCS roster changes (LTA North counts as LCS), with team and player redirects resolved, staff and Game Changers rows skipped, and the full history of in-scope players kept.
- Ingestor: idempotent upserts into Neon, soft deletion of events removed from the wiki, incremental runs with a 14-day overlap, `--full` two-season backfill and `--since`; each run is recorded in `sync_state`.
- CI: Postgres service for the ingestor database test.
- Fixtures from Leaguepedia in `ingestor/tests/fixtures/probe/` (CC BY-SA).

### Changed

- Ingest workflow timeout raised from 15 to 30 minutes so a full backfill fits.

## [0.1.0] - 2026-11-18

### Added

- Initial project setup: pnpm workspace with the Vue 3 SPA, Vercel read-only API, shared types and the Rust ingestor skeleton.
- CI workflow with lint, format, typecheck, build and Rust checks; manual ingestion workflow.
- Neon Postgres database with separate read-only and read/write roles.

[Unreleased]: https://github.com/PabloViniegra/lol-market-changes/compare/056d672...HEAD
[0.1.0]: https://github.com/PabloViniegra/lol-market-changes/commit/056d672
