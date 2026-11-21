-- Every official move, tracked league or not. The player page needs moves outside the four
-- leagues to explain where a player is now; the feed keeps filtering them out.
create view official_moves as
select
  t.id::text as id,
  'official'::text as status,
  t.player,
  t.from_team,
  t.to_team,
  league_at(t.from_team, t.date) as from_league,
  league_at(t.to_team, t.date) as to_league,
  t.role,
  t.kind,
  t.date,
  null::smallint as reliability,
  (
    select coalesce(jsonb_agg(distinct jsonb_build_object('url', e.source_url)), '[]'::jsonb)
    from raw_events e
    where e.id in (t.leave_event, t.join_event) and e.source_url is not null
  ) as sources,
  null::text as notes,
  (select r.id from rumors r where r.transfer_id = t.id order by r.reported_at limit 1) as rumor_id,
  null::text as transfer_id
from transfers t;

-- Same columns as before, so `create or replace` keeps the view and its grants.
create or replace view market_feed as
select *
from official_moves
where from_league is not null or to_league is not null
union all
select
  r.id,
  r.status,
  r.player,
  r.from_team,
  r.to_team,
  league_at(r.from_team, r.reported_at),
  league_at(r.to_team, r.reported_at),
  r.role,
  r.kind,
  r.reported_at,
  r.reliability,
  r.sources,
  r.notes,
  null,
  r.transfer_id::text
from rumors r;
