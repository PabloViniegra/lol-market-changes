create table teams (
  name          text primary key,
  short         text,
  league        text not null default 'other' check (league in ('LCK', 'LPL', 'LEC', 'LCS', 'other')),
  region        text,
  org           text,
  logo_url      text
);

create table team_seasons (
  team          text not null references teams (name),
  league        text not null check (league in ('LCK', 'LPL', 'LEC', 'LCS')),
  tournament    text not null,
  date_start    date not null,
  date_end      date,
  primary key (team, tournament)
);

create table players (
  name          text primary key,
  real_name     text,
  country       text,
  role          text,
  image_url     text
);

create table raw_events (
  id            text primary key,
  date          date not null,
  player        text not null references players (name),
  player_raw    text not null,
  team          text not null references teams (name),
  team_raw      text not null,
  direction     text not null check (direction in ('join', 'leave')),
  role          text,
  preload       text not null,
  status        text,
  is_sub        boolean not null default false,
  is_loan       boolean not null default false,
  is_retired    boolean not null default false,
  news_id       text,
  line_in_news  int,
  source_url    text,
  fetched_at    timestamptz not null default now(),
  deleted_at    timestamptz
);

create index raw_events_player_date on raw_events (player, date);
create index raw_events_date on raw_events (date);

create table sync_state (
  id                 int primary key default 1 check (id = 1),
  last_synced_date   date,
  last_run_at        timestamptz,
  last_run_status    text check (last_run_status in ('ok', 'error')),
  stats              jsonb
);
