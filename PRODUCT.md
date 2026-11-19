# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

The owner, a close follower of competitive League of Legends, possibly shared informally with a few friends. They check in several times a day during the offseason transfer window to see what moved, who is rumored where, and which teams are rebuilding. No accounts and no public-scale audience.

## Product Purpose

A dashboard of the LoL esports transfer market for LCK, LPL, LEC and LCS. It shows official roster moves from Leaguepedia, paired into transfers (from team to team) instead of raw join/leave events, next to hand-curated rumors, each clearly labelled with its status. Success means answering "who moved, from where, to where, and how sure is it" at a glance, plus "what moved this week in a league" and "did that rumor come true".

## Positioning

Leaguepedia records events; news sites scatter rumors. This product pairs events into transfers, puts rumors in the same feed, and reconciles rumors against official moves automatically, so a rumor's outcome (confirmed, denied, stale) is tracked over time.

## Operating Context

- Official moves are ingested from Leaguepedia every 15 minutes by a GitHub Actions job; data can lag up to about 30 minutes.
- Rumors are added by editing `data/rumors.yaml` and committing.
- The frontend is a read-only SPA over a read-only API (`/api/meta`, `/api/feed`, `/api/stats`, `/api/flows`, `/api/teams/:name`, `/api/players/:name`).
- Views: dashboard, market feed, flows, team, player, rumor tracker.

## Capabilities and Constraints

- Leagues: LCK, LPL, LEC, LCS. A move is in scope if either team was in one of them on the move date.
- Move kinds: transfer, free agent signing, release, loan, promotion, bench, sub, retirement. Players only; staff is out of scope.
- Statuses: official, rumor, confirmed, denied, stale.
- Filters (league, team, role, kind, status, dates) are mirrored in the URL.
- Freshness from `/api/meta`; a warning when the last sync is older than one hour.
- Desktop first, usable on mobile. Dark theme only in v1.
- Runs on free tiers (€0/month). No real-time push, user accounts, notifications or contract data.
- UI copy is in English.

## Brand Commitments

- Leaguepedia data is CC BY-SA: the UI must keep the attribution footer linking to Leaguepedia.
- Team logos and player photos from Leaguepedia only identify a team or player next to its code or name, never as decoration. No Riot or league brand marks.

## Evidence on Hand

Live data in Neon via the API (moves, rosters, rumors). No testimonials, users or metrics to cite.

## Product Principles

- Data first: the moves are the headline, chrome stays quiet.
- Status is never ambiguous: every item says whether it is official, rumored, confirmed, denied or stale.
- Fast to scan for someone checking several times a day.
- Shareable state: what you see is reproducible from the URL.

## Accessibility & Inclusion

WCAG AA contrast, visible keyboard focus and full keyboard reachability of rows, filters and charts; status and league never conveyed by colour alone; reduced-motion respected.
