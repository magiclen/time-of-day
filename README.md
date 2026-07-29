Time of Day
====================

[![CI](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml)

A Rust workspace for representing times from `00:00` through the distinct end-of-day boundary `24:00`, with optional integrations for common time libraries and databases.

## Crates

- [time-of-day](time-of-day/README.md) provides the core typed and dynamic time-of-day values.
- [time-of-day-sqlx](time-of-day-sqlx/README.md) provides SQLx adapters for PostgreSQL, MySQL, MariaDB, and SQLite.
- [time-of-day-tiberius](time-of-day-tiberius/README.md) provides Tiberius storage policies for SQL Server.

## License

Each crate is available under the MIT License.
