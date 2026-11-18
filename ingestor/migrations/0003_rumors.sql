create table rumors (
  id            text primary key,
  player        text not null,
  from_team     text,
  to_team       text,
  role          text,
  kind          text,
  reported_at   date not null,
  reliability   smallint not null check (reliability between 1 and 5),
  sources       jsonb not null,
  notes         text,
  status        text not null check (status in ('rumor', 'confirmed', 'denied', 'stale')),
  -- Transfers are rebuilt from raw events, so a link can vanish; the next reconcile relinks it.
  transfer_id   uuid references transfers (id) on delete set null,
  updated_at    timestamptz not null default now()
);

create index rumors_status_reported_at on rumors (status, reported_at desc);
