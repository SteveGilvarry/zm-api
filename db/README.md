# ZoneMinder schema files

The `*.sql` files in this directory are **vendored, unmodified copies from the
[ZoneMinder](https://github.com/ZoneMinder/zoneminder) project** and remain licensed
under ZoneMinder's **GPL-2.0** license — not under zm_api's AGPL-3.0 / commercial license.

They are the schema fragments that `zm_create.sql.in` pulls in via `source` directives.
The test-database setup scripts (`scripts/setup-ci-db.sh`, `scripts/db-manager.sh`)
resolve those directives by inlining these files when building the test schema.

| File | Purpose |
|------|---------|
| `Object_Types.sql` | `Object_Types` table |
| `User_Preferences.sql` | `User_Preferences` table |
| `triggers.sql` | X10 trigger seed data |
| `manufacturers.sql` | Camera manufacturer seed data |
| `models.sql` | Camera model seed data |

Source: <https://github.com/ZoneMinder/zoneminder/tree/master/db>
