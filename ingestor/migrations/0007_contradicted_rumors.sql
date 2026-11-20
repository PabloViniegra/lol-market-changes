alter table rumors drop constraint rumors_status_check;
alter table rumors add constraint rumors_status_check
  check (status in ('rumor', 'confirmed', 'denied', 'stale', 'contradicted'));
