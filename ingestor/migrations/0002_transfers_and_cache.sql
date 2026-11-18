alter table teams add column renamed_to text;

create table transfers (
  id            uuid primary key default gen_random_uuid(),
  player        text not null references players (name),
  from_team     text references teams (name),
  to_team       text references teams (name),
  role          text,
  kind          text not null check (kind in ('transfer', 'free_agent_signing', 'release', 'loan',
                  'promotion', 'bench', 'sub', 'retirement')),
  date          date not null,
  leave_event   text references raw_events (id),
  join_event    text references raw_events (id),
  updated_at    timestamptz not null default now(),
  check (leave_event is not null or join_event is not null),
  unique nulls not distinct (leave_event, join_event)
);

create index transfers_player on transfers (player);
create index transfers_date on transfers (date desc);
create index transfers_from_team on transfers (from_team);
create index transfers_to_team on transfers (to_team);

create table leaguepedia_cache (
  name          text primary key,
  since         date not null,
  rows          jsonb not null,
  fetched_at    timestamptz not null default now()
);
