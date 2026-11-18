alter table team_seasons add column roster jsonb not null default '[]';

-- The cached scope rows predate RosterLinks and Roles; dropping them forces a refetch.
delete from leaguepedia_cache where name = 'scope';

-- League of a team on a date, from tournament rosters. A team counts from 90 days before a
-- split starts (offseason signings) until a year after it ends (post-season releases).
create function league_at(team text, d date) returns text
language sql stable
as $$
  select s.league
  from team_seasons s
  where s.team = $1
    and s.date_start <= $2 + 90
    and coalesce(s.date_end, s.date_start) >= $2 - 365
  order by s.date_start desc
  limit 1
$$;

-- Official moves where either side was in a tracked league at the time, plus every rumor.
create view market_feed as
select *
from (
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
  from transfers t
) official
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
