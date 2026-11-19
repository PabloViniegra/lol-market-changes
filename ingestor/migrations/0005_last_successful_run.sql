alter table sync_state add column last_successful_run_at timestamptz;

update sync_state
set last_successful_run_at = last_run_at
where last_run_status = 'ok';
