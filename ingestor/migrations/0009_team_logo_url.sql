-- The ingestor resolves each team's square logo through file redirects; null when the wiki has none.
alter table teams add column logo_url text;
