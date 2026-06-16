# Grebball-Discord

A Discord application/bot for running an NFL prediction pool — replacing an
older custom web app for collecting football predictions and showing results.

The project has two parts that share a single SQLite database:

| Part | Stack | Location | Role |
| --- | --- | --- | --- |
| **Bot** | Rust + [Serenity](https://github.com/serenity-rs/serenity) | `src/`, `Cargo.toml` | Slash commands, scoring, posting results |
| **Web pick app** | Node.js + Express + EJS | `web/` | Web page where poolers submit their weekly picks |

Both read/write `local/local.db`, so you only ever create **one** database file.

## Prerequisites

- **Rust** (stable) + Cargo — https://rustup.rs
- **Node.js** 18+ and npm
- **SQLite 3** command-line tool (`sqlite3`) to create the database
- A **Discord application + bot token** (see [Discord setup](#discord-setup))

> On Windows the repo ships `sqlite3.dll` / `sqlite3.lib` for building, but you
> still need the separate `sqlite3.exe` CLI to run the schema. Grab it from the
> [SQLite download page](https://www.sqlite.org/download.html) (the
> "sqlite-tools" bundle) and put it on your `PATH`.

## 1. Configure environment variables

The bot loads configuration from a `.env` file at the repo root (via the
`dotenv` crate). Start from the example:

```sh
cp .env.example .env
```

Then fill in your own values. `.env` is gitignored, so your secrets stay local.
The variables you must set are:

| Variable | Required | What it is |
| --- | --- | --- |
| `DISCORD_TOKEN` | yes | Your bot token (Developer Portal → Bot → Reset Token). |
| `GUILD_ID` | yes | The server the bot registers slash commands to. |
| `RESULTS_WEBHOOK` | yes | Discord webhook the bot posts results to. |
| `POOL_ID` | yes | Which pool row (in `pools`) this instance manages. |
| `CONF_SEASON` | yes | The NFL season year, e.g. `2025`. |
| `DATABASE_URL` | yes | SQLite URL — leave as `sqlite:local/local.db`. |
| `PICKS_URL` | yes | Base URL of the web pick app (`http://localhost:3000` locally). |
| `DATA_URL` / `STANDINGS_URL` | yes | ESPN scoreboard / standings endpoints (defaults provided). |
| `WEEKLY_WEBHOOK` | optional | Webhook for the startup weekly reminder. |
| `BLAME_URL` | optional | Only needed by the `/blame` command. |

## 2. Create your local database

The database lives in `local/` (gitignored). Create the folder and build a
fresh database from the schema:

```sh
mkdir local
sqlite3 local/local.db < db/struct-features-capsules.sql
```

> Use `db/struct-features-capsules.sql` — it is the current, complete schema
> (users, pools, poolers, picks, match_picks, features, capsules, pick_tokens).
> The older `db/struct.sql` is kept for history only and is missing tables the
> code expects.

You now need at least one **pool** and the **poolers** in it before the bot can
do anything useful. A minimal seed:

```sh
sqlite3 local/local.db
```
```sql
INSERT INTO pools (name, motp) VALUES ('My Test Pool', NULL);
-- note the pool id (likely 1) and use it as POOL_ID in your .env
INSERT INTO poolers (name, favteam, poolid) VALUES ('Alice', 'NE', 1);
INSERT INTO poolers (name, favteam, poolid) VALUES ('Bob', 'DAL', 1);
.quit
```

## 3. Run the bot

From the repo root:

```sh
cargo run
```

On startup the bot registers its slash commands to your `GUILD_ID` and connects
to Discord. Guild-scoped commands appear almost immediately.

## 4. Run the web pick app

In a second terminal:

```sh
cd web
npm install
node app.js
```

It listens on **http://localhost:3000** (matching `PICKS_URL`). The bot links
poolers to this page when they go to submit picks.

## Discord setup

You'll want your own bot + a throwaway server so you never touch production:

1. **Create the application** at the
   [Discord Developer Portal](https://discord.com/developers/applications) →
   *New Application*.
2. **Add a bot**: *Bot* tab → *Add Bot*. Click *Reset Token* and copy it into
   `DISCORD_TOKEN`. Keep this secret.
3. **Privileged intents**: this bot uses the default gateway intents, so you can
   leave the privileged toggles off unless you add features that need them.
4. **Invite the bot**: *OAuth2 → URL Generator*, select the `bot` and
   `applications.commands` scopes, pick permissions (Send Messages is enough to
   start), open the generated URL and add the bot to your test server.

### Creating a personal test server

1. In Discord, click the **+** in the server list → *Create My Own* → *For me
   and my friends*.
2. Enable **Developer Mode**: *User Settings → Advanced → Developer Mode*.
3. Right-click the server icon → **Copy Server ID** and paste it into
   `GUILD_ID`.
4. For `RESULTS_WEBHOOK` (and optionally `WEEKLY_WEBHOOK`): right-click a
   channel → *Edit Channel → Integrations → Webhooks → New Webhook*, then
   *Copy Webhook URL*.

That gives you a fully isolated environment to develop against.

## Project layout

```
src/                 Rust bot
  main.rs            entry point, command registration, startup tasks
  commands/          one module per slash command
  database.rs        SQLite access layer (sqlx)
  football.rs        ESPN / data-source integration and scoring
db/                  SQL schema files
  struct-features-capsules.sql   <- current schema
local/               SQLite database (gitignored, you create this)
web/                 Express pick app
  app.js             server + routes (port 3000)
  database.js        opens ../local/local.db
  views/             EJS/HTML templates
.env.example         template for your .env
```
