# Fixtures

Real responses from the [Leaguepedia](https://lol.fandom.com/) Cargo API, licensed [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).

`probe/` holds one month of LEC and LCK roster changes plus the related teams, players, player images, redirects and tenures, plus the square-logo file each team resolves to (`logo_files.json`, rows built from the MediaWiki `action=query` response), written by `cargo run --bin probe` (see `probe/meta.json` for the date range). Each file is the list of `cargoquery[].title` rows, unmodified.
