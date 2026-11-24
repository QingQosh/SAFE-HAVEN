# Database Migrations

This project uses [`sqlx`](https://github.com/launchbadge/sqlx) migrations to manage
schema changes. Migrations are versioned SQL files stored in `migrations/` and are
applied in filename order, so every environment converges on the same schema.

## Layout

```
migrations/
  20240101000000_create_users.sql
  20240102000000_add_sessions.sql
```

Each file is named `<version>_<description>.sql`. The version prefix is a UTC
timestamp (`YYYYMMDDHHMMSS`) so ordering is deterministic across branches.

## Running migrations

Migrations run automatically on deployment via the application startup path
(`sqlx::migrate!()`), so no manual step is required. To run them by hand:

```sh
sqlx migrate run
```

To check which migrations are pending or already applied:

```sh
sqlx migrate info
```

`sqlx` records applied versions in the `_sqlx_migrations` table, which is how
migration status is tracked and how environments stay in sync.

## Creating a migration

```sh
sqlx migrate add <description>
```

This creates a new timestamped file. Keep migrations small and additive
(add columns/tables/indexes) so minor changes can be applied without downtime.

## Rollback

Recent migrations can be reverted with:

```sh
sqlx migrate revert
```

This reverts the most recently applied migration. For a migration to be
reversible, include a `-- Add down migration script here` section (or a paired
`*.down.sql` file) with the inverse statements. Rollback is intended for recent
migrations only; destructive changes should be reviewed before reverting in
production.

## Zero-downtime guidance

- Prefer additive changes (new nullable columns, new tables, new indexes).
- Backfill data in a separate step before adding `NOT NULL` constraints.
- Drop old columns/tables only after the code that used them is deployed.

## Environments

Dev, staging, and prod all run the same migration set on deploy. Test new
migrations in staging first, then promote to prod.
