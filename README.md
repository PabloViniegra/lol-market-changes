<div align="center">
  <img src="web/public/logo.webp" alt="" width="96" height="96">
  <h1>Transfer Board</h1>
  <p><strong>The League of Legends esports transfer market at a glance</strong></p>
  <p>LCK · LPL · LEC · LCS</p>
  <p>
    <img alt="Vue 3" src="https://img.shields.io/badge/Vue-3-237451?logo=vuedotjs&amp;logoColor=white">
    <img alt="TypeScript 7" src="https://img.shields.io/badge/TypeScript-7-3178C6?logo=typescript&amp;logoColor=white">
    <img alt="Rust 2024 edition" src="https://img.shields.io/badge/Rust-2024-000000?logo=rust&amp;logoColor=white">
    <img alt="Neon Postgres" src="https://img.shields.io/badge/Neon-Postgres-177057?logo=postgresql&amp;logoColor=white">
    <img alt="Vercel" src="https://img.shields.io/badge/Hosted_on-Vercel-000000?logo=vercel&amp;logoColor=white">
  </p>
  <p>
    <strong><a href="https://lol-market-changes.vercel.app/">Open the live board</a></strong> ·
    <a href="https://lol-market-changes.vercel.app/feed">Browse the feed</a> ·
    <a href="https://lol-market-changes.vercel.app/rumors">Track rumors</a>
  </p>
</div>

Transfer Board pairs Leaguepedia roster events into player moves so you can see where each player came from and where they went. Curated rumors appear alongside official moves and are marked confirmed when a matching move appears. The dashboard shows when the data last synced.

[Features](#features) · [Architecture](#architecture) · [Local development](#local-development) · [Configuration](#configuration) · [Checks](#checks) · [Data and license](#data-and-license)

## Features

| View              | What you can explore                                                                                                                          |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Dashboard         | Latest moves, activity by league, free agents, busiest teams, and an interactive diagram of player flows between leagues or teams.            |
| Market feed       | Search by player and filter by league, team, role, move type, status, or date. The URL preserves your filters so you can share the same view. |
| Teams and players | Compare current and previous team rosters, review arrivals and departures, or follow a player's move and rumor history.                       |
| Rumor tracker     | Follow open, confirmed, denied, and stale reports with source links and reliability ratings.                                                  |

The official feed covers player moves where either team belonged to a tracked league on the move date.

## Architecture

```mermaid
flowchart LR
    A[Leaguepedia roster events] --> C[Rust ingestor]
    B[Curated rumors] --> C
    C -->|Upserts| D[(Neon Postgres)]
    D -->|Market data| E[Vercel API]
    E --> F[Vue dashboard]
```

[GitHub Actions](.github/workflows/ingest.yml) schedules ingestion every 15 minutes. The Rust ingestor pairs roster events, syncs [`data/rumors.yaml`](data/rumors.yaml), and checks for official moves that confirm open rumors. It is the only database writer. The public API connects with a read-only Postgres role. The project is designed to run on free tiers.

Readers can submit a rumor for email review. Approval commits the reviewed report and its public source to `data/rumors.yaml` and triggers ingestion. Private notes stay in the review email. [Send a rumor](https://lol-market-changes.vercel.app/rumors/new)

| Path                                   | Responsibility                                                                      |
| -------------------------------------- | ----------------------------------------------------------------------------------- |
| [`web/`](web/)                         | Vue 3 dashboard with Vite, Tailwind CSS, Vue Router, and TanStack Query.            |
| [`api/`](api/)                         | TypeScript Vercel functions for market data and rumor moderation.                   |
| [`ingestor/`](ingestor/)               | Rust batch job for Leaguepedia imports, transfer pairing, and rumor reconciliation. |
| [`packages/shared/`](packages/shared/) | TypeScript response types shared by the API and frontend.                           |

## Local development

Use Node.js 24 and pnpm 11.2.2, as specified in [`package.json`](package.json). For the ingestor, use Rust stable with `rustfmt` and `clippy` from [`rust-toolchain.toml`](rust-toolchain.toml).

### Frontend with live data

```sh
pnpm install
pnpm dev
```

Open `http://localhost:5173`. Vite proxies `/api` to the live site by default. Set `API_ORIGIN` when starting Vite to use a different API.

### Frontend and API together

Use the Vercel CLI with access to the linked project. Run these commands from the repository root after installing the workspace dependencies. [Vercel CLI](https://vercel.com/docs/cli)

```sh
vercel link
vercel env pull .env.local
vercel dev
```

### Ingestor

The dry run fetches and normalizes Leaguepedia data and validates the rumors file without connecting to Postgres. The incremental sync and backfill require `INGESTOR_DATABASE_URL` in the shell environment.

```sh
cd ingestor
cargo run -- --help
cargo run -- --dry-run
cargo run                   # Incremental sync
cargo run -- --full         # Backfill the last two seasons
cargo run --bin probe       # Refresh Leaguepedia JSON fixtures
```

## Configuration

See [`.env.example`](.env.example) for placeholders.

| Variable                | Purpose                                                                                                                                                       |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DATABASE_URL`          | Pooled Neon connection for the API's read-only Postgres role. Set in Vercel.                                                                                  |
| `INGESTOR_DATABASE_URL` | Read/write Neon connection. Set as a GitHub Actions secret for scheduled runs or export it for local runs.                                                    |
| `RESEND_API_KEY`        | Resend credential for review emails sent from `send.pabloviniegra.dev`. Set in Vercel.                                                                        |
| `REVIEW_EMAIL_TO`       | Inbox for submitted rumors. Set in Vercel.                                                                                                                    |
| `REVIEW_SECRET`         | Secret of at least 32 characters used to protect review links. Set in Vercel.                                                                                 |
| `GITHUB_TOKEN`          | Fine-grained token scoped to this repository with read/write access to Contents and Actions for committing approvals and triggering ingestion. Set in Vercel. |

<details>
<summary>Moderation and privacy operations</summary>

New review links are encrypted and expire after 14 days. Previously issued signed links work until their original expiry. The Vercel Firewall limits submissions to three requests per IP every ten minutes.

- Confirm that the source opens the public report. Remove tracking parameters, replace any credential-bearing URL, and decline reports containing unrelated personal details.
- Remove review emails and copies when no longer needed. Check pending mail when links expire and document any reason for keeping a message. Confirm provider retention, processing terms, and transfer safeguards against the [privacy notice](https://lol-market-changes.vercel.app/privacy).
- Handle privacy requests through the contact in the notice within one month and verify identity only as needed. Correct the upstream Leaguepedia record where applicable so later syncs preserve the correction. For a submitted rumor, update or remove it in `data/rumors.yaml`, run ingestion, and verify the site and API. Review public Git history and third-party copies when relevant to the request.

</details>

## Checks

Run the same web and Rust checks as [CI](.github/workflows/ci.yml).

```sh
pnpm check

cd ingestor
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

`pnpm check` runs lint, formatting checks, TypeScript and Vue type checks, API tests, and the web build. CI also provides PostgreSQL 17 for the database integration test. To run it locally, point `TEST_DATABASE_URL` at a disposable database, since the test truncates ingestor tables.

```sh
docker run --rm -d --name lol-market-pg -e POSTGRES_PASSWORD=dev -e POSTGRES_DB=lol_market -p 127.0.0.1:55432:5432 postgres:17
```

Once PostgreSQL is ready, run from `ingestor/`.

```sh
TEST_DATABASE_URL=postgres://postgres:dev@127.0.0.1:55432/lol_market cargo test --locked
docker stop lol-market-pg
```

## Data and license

Official roster data comes from Leaguepedia and its contributors. Transfer Board normalizes names and groups related roster events into transfers. The source data and those adaptations are licensed under CC BY-SA 3.0. The software is licensed under MIT. [Leaguepedia attribution](https://lol.fandom.com/wiki/Leaguepedia:Copyrights) · [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) · [MIT license](LICENSE)

Transfer Board is an independent project. Riot Games has not endorsed it, and it does not represent Riot Games or anyone officially involved in creating or managing its properties. Riot Games and associated properties are trademarks or registered trademarks of Riot Games, Inc.

[Release notes](CHANGELOG.md) · [Design system](DESIGN.md) · [Data and credits](https://lol-market-changes.vercel.app/credits)
