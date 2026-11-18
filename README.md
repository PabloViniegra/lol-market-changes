# LoL Market Changes

![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-7-3178C6?logo=typescript&logoColor=white)
![Vue](https://img.shields.io/badge/Vue-3-4FC08D?logo=vuedotjs&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-8-646CFF?logo=vite&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-4-06B6D4?logo=tailwindcss&logoColor=white)
![Neon](https://img.shields.io/badge/Neon-Postgres-00E599?logo=postgresql&logoColor=white)
![Vercel](https://img.shields.io/badge/Vercel-deployed-000000?logo=vercel&logoColor=white)

A dashboard for the League of Legends esports transfer market across LCK, LPL, LEC and LCS. It pairs official roster events from [Leaguepedia](https://lol.fandom.com/) into transfers (from → to), tracks hand-curated rumors next to them, and reconciles rumors once they become official.

**Status:** initial setup. The repository builds and deploys, but no data is ingested yet. Live site: <https://lol-market-changes.vercel.app>

- [Architecture](#architecture)
- [Repository layout](#repository-layout)
- [Requirements](#requirements)
- [Getting started](#getting-started)
- [Checks](#checks)
- [Configuration](#configuration)
- [Roadmap](#roadmap)
- [Attribution](#attribution)

## Architecture

```
GitHub Actions ──► Rust ingestor ──upsert──► Neon Postgres ◄──read── Vercel functions ◄── Vue SPA
                   (Leaguepedia + rumors.yaml)                       (read-only role)
```

Everything runs on free tiers. The ingestor is the only writer; the public API uses a read-only Postgres role.

## Repository layout

| Path                 | Description                                                             |
| -------------------- | ----------------------------------------------------------------------- |
| `ingestor/`          | Rust batch job: fetch, normalise, pair events, sync rumors, reconcile.  |
| `api/`               | Vercel serverless functions in TypeScript, one file per endpoint.       |
| `web/`               | Vue 3 single-page app (Vite, Tailwind CSS, Vue Router, TanStack Query). |
| `packages/shared/`   | Response types shared by `api/` and `web/`.                             |
| `data/rumors.yaml`   | Rumors, edited by hand and committed.                                   |
| `.github/workflows/` | `ci.yml` (checks) and `ingest.yml` (ingestion, manual trigger for now). |

## Requirements

- Node.js 24 and pnpm 11 (`corepack enable`)
- Rust stable with `rustfmt` and `clippy` (pinned in `rust-toolchain.toml`)
- [Vercel CLI](https://vercel.com/docs/cli), only to run the API locally

## Getting started

```sh
pnpm install
pnpm dev                   # SPA at http://localhost:5173
```

To serve the SPA and the API together against the database:

```sh
vercel link
vercel env pull .env.local
vercel dev
```

Run the ingestor:

```sh
cd ingestor
cargo run -- --help
cargo run -- --dry-run
```

## Checks

These are the same checks CI runs:

```sh
pnpm check                 # oxlint, oxfmt --check, tsc / vue-tsgo, vite build

cd ingestor
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Configuration

See [`.env.example`](.env.example).

| Variable                | Used by                                                        | Role                   |
| ----------------------- | -------------------------------------------------------------- | ---------------------- |
| `DATABASE_URL`          | `api/` (Vercel env)                                            | Read-only role.        |
| `INGESTOR_DATABASE_URL` | Ingestor (GitHub secret in `ingest.yml`, `.env.local` locally) | Read/write owner role. |

## Roadmap

1. Data probe: confirm the Leaguepedia Cargo tables and fields.
2. Ingestor core: fetch, normalise and upsert raw events, plus migrations.
3. Pairing of events into transfers.
4. Rumors: schema, validation and reconciliation.
5. Scheduled ingestion every 15 minutes.
6. Read API: feed, stats, flows, teams and players.
7. Frontend: dashboard, feed, flows, team, player and rumor tracker views.

## Attribution

Official roster data comes from [Leaguepedia](https://lol.fandom.com/) and is licensed [CC BY-SA](https://creativecommons.org/licenses/by-sa/3.0/).

LoL Market Changes isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
