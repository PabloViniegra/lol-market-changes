# AGENTS.md

LoL esports transfer market dashboard: official roster moves from Leaguepedia plus hand-curated rumors, for LCK, LPL, LEC and LCS. Runs entirely on free tiers (€0/month).

The product spec lives in `docs/PRD.md` (git-ignored, local only). Read the relevant section before changing behaviour.

## Layout

| Path               | What                                                                                                                 | Stack                                                   |
| ------------------ | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `ingestor/`        | Batch job run by GitHub Actions: fetch Leaguepedia → normalise → pair events → sync rumors → reconcile → upsert Neon | Rust 2024, standalone crate                             |
| `api/`             | Read-only Vercel functions, one file per endpoint (`api/meta.ts` → `/api/meta`)                                      | TypeScript, `@neondatabase/serverless`                  |
| `web/`             | SPA                                                                                                                  | Vue 3, Vite, Tailwind CSS 4, Vue Router, TanStack Query |
| `packages/shared/` | API response types shared by `api/` and `web/` (type-only imports)                                                   | TypeScript                                              |
| `data/rumors.yaml` | Rumors, edited by hand and committed                                                                                 | YAML                                                    |

## Commands

From the repo root (pnpm workspace):

```sh
pnpm install
pnpm dev            # web dev server
pnpm check          # lint + format check + typecheck + build
pnpm format         # oxfmt write
```

From `ingestor/`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --dry-run
```

CI (`.github/workflows/ci.yml`) runs exactly these. Keep both green.

## Rules

- **pnpm only.** Never add `package-lock.json` or `yarn.lock`.
- **TypeScript 7 is mandatory.** Type-check `.ts` with `tsc` and `.vue` with `vue-tsgo`. Do not add `vue-tsc` or downgrade `typescript`: `vue-tsc` needs the TS ≤6 JS API.
- **oxlint + oxfmt** are the only linter and formatter. Do not add ESLint or Prettier. Lint runs with `--deny-warnings`.
- `web/` and `api/` share types only through `@lol/shared`, using `import type`. `api/` must not import from `web/`, nor `web/` from `api/`.
- **The API is read-only.** It uses the Postgres read-only role (`DATABASE_URL` in Vercel). Only the ingestor writes, using the read/write role via `INGESTOR_DATABASE_URL` (GitHub secret, `.env.local` locally). Never point `DATABASE_URL` at the read/write role.
- Ingestor: keep the domain logic (pairing, rumor reconciliation, normalisation) as pure functions over plain data, tested with real Leaguepedia fixtures in `ingestor/tests/fixtures/`. Keep HTTP and DB code at the edges.
- The ingestor must be idempotent: two consecutive runs produce no changes.
- Leaguepedia etiquette: ≤ 1 request/s, an identifiable User-Agent, incremental fetches. Data is CC BY-SA, so the UI must keep the attribution footer.
- Never commit secrets. `.env*` files are git-ignored. Use `vercel env pull` locally.
- Ask before adding a dependency the PRD does not already list.
- Record every user-visible or operational change under `## [Unreleased]` in `CHANGELOG.md` ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/) sections: Added, Changed, Deprecated, Removed, Fixed, Security) in the same commit. On release, rename it to `## [x.y.z] - YYYY-MM-DD` and update the compare links.

## Skills

Project skills are installed in both `.claude/skills/` and `.agents/skills/` (keep them in sync; `skills-lock.json` pins their versions):

- `neon-postgres`: Neon connection and serverless driver.
- `vue-pinia-best-practices`: Vue and Pinia state.
- `tanstack-query-best-practices`: server state in `web/`.
- `echarts`: Sankey, chord and bar charts.

Restore them with `npx skills experimental_install`.

<!-- graphify:start -->

## Graphify

- Global agent instructions take precedence over this generated guidance.
- Use Graphify for cross-file relationship or impact questions when an existing graph is available. Direct reads are appropriate for known files and small scoped edits.
- From the repository root, use `python -m graphify query "<question>" --budget 800`, `path`, `explain`, or `affected` as appropriate.
- If the graph is missing, stale, or unhelpful, use direct search. Do not repeat an unavailable-graph check within the same task.
- Build, update, or install Graphify only when requested or when graph maintenance is part of the task.

<!-- graphify:end -->
