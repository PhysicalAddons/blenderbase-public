
### Commands (https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md):
- cargo install sqlx-cli --no-default-features --features sqlite
- cargo install sqlx-cli --no-default-features --features sqlite-unbundled
- sqlx database create
- sqlx database drop
- sqlx migrate add -r <name>
- sqlx migrate run
- sqlx migrate revert

If migrations are not failing but running dev gives a migration mistmatch, then delete all .db files in AppData local/roaming.