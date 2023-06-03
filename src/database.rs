use std::env;

use anyhow::{Result, anyhow};

use sqlx::{ Pool, Sqlite, Row };
use sqlx::sqlite::SqlitePool;

pub struct DB {
    pool: Pool<Sqlite>,
}

impl DB {
    pub async fn new() -> DB {
        let db_url = env::var("DATABASE_URL")
            .expect("![MAIN] Cannot find 'DATABASE_URL' in env");

        DB { pool: SqlitePool::connect(db_url.as_str()).await.unwrap() }
    }

    pub async fn fetch_results(&self, discordid: &i64, week: &i64) -> Result<()> {
        println!("[DB] Getting results for discordid: {discordid}, week {week}");
        //TODO: Finish implementation

        let users = sqlx::query("
                SELECT id, season, week, pickstring, poolerid, scorecache FROM picks
                ")
            .fetch_all(&self.pool)
            .await?;

        for row in users {
            let id: i64 = row.get("id");
            let season: i64 = row.get("season");
            let week: i64 = row.get("week");
            let picks: Option<String> = row.get("pickstring");
            let poolerid: i64 = row.get("poolerid");
            let score: Option<i64> = row.get("scorecache");

            println!("|{}|{}|{}|{:?}|{}|{:?}|", id, season, week, picks, poolerid, score);
        }

        Ok(())
    }

    pub async fn prime_picks(&self, discordid: &i64, week: &i64) -> Result<i64> {
        let season = env::var("CONF_SEASON")
            .expect("[DB] Cannot find 'CONF_SEASON' in env").parse::<u16>()
            .expect("[DB] Could not parse 'CONF_SEASON' to u16");

        let poolerid: i64 = sqlx::query("
                SELECT p.id FROM users AS u
                JOIN poolers AS p
                ON u.id = p.userid
                WHERE u.discordid = ?
                ")
            .bind(discordid)
            .fetch_one(&self.pool)
            .await?
            .get("id");

        match sqlx::query("
                SELECT id, pickstring FROM picks
                WHERE poolerid = ? AND season = ? AND week = ?
                ")
            .bind(poolerid)
            .bind(season)
            .bind(week)
            .fetch_one(&self.pool)
            .await {
                Ok(row) => {
                    if let Some(_) = row.get::<Option<String>, &str>("pickstring") {
                        return Err(anyhow!("[DB] Picks for given pooler, season and week already entered!"));
                    }
                    else {
                        // Picks are already primed and not filled
                        let id: i64 = row.get("id");
                        return Ok(id);
                    }
                },
                Err(_) => {
                    let new_row = sqlx::query("
                            INSERT INTO picks (season, week, poolerid)
                            VALUES (?, ?, ?);
                            SELECT last_insert_rowid();
                            ")
                        .bind(season)
                        .bind(week)
                        .bind(poolerid)
                        .fetch_one(&self.pool)
                        .await?;

                    let id: i64 = new_row.get(0);
                    Ok(id)
                }
        }
    }
}
