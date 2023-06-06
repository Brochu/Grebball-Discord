use std::env;

use anyhow::{Result, anyhow};

use sqlx::{ Pool, Sqlite, Row };
use sqlx::sqlite::SqlitePool;

pub struct DB {
    pool: Pool<Sqlite>,
}

//TODO: Adding a struct to hold results

impl DB {
    pub async fn new() -> DB {
        let db_url = env::var("DATABASE_URL")
            .expect("![MAIN] Cannot find 'DATABASE_URL' in env");

        DB { pool: SqlitePool::connect(db_url.as_str()).await.unwrap() }
    }

    pub async fn fetch_results(&self, discordid: &i64, week: &i64) -> Result<Vec<(String, u32)>> {
        let poolid: i64 = sqlx::query("
                SELECT p.poolid FROM users AS u
                JOIN poolers AS p
                ON u.id = p.userid
                WHERE u.discordid = ?
                ")
            .bind(discordid)
            .fetch_one(&self.pool)
            .await?
            .get("poolid");

        let season = env::var("CONF_SEASON")
            .expect("[DB] Cannot find 'CONF_SEASON' in env").parse::<u16>()
            .expect("[DB] Could not parse 'CONF_SEASON' to u16");

        let picks = sqlx::query("
                SELECT name, pickstring, scorecache FROM picks AS pk
                JOIN poolers AS pl ON pl.id = pk.poolerid
                WHERE season = ? AND week = ? AND pl.poolid = ?
                ")
            .bind(season)
            .bind(week)
            .bind(poolid)
            .fetch_all(&self.pool)
            .await?;

        let results: Vec<(String, u32)> = picks.iter()
            .map(|row| {
                let name: String = row.get("name");
                if let Some(cached) = row.get::<Option<u32>, &str>("scorecache") {
                    (name, cached)
                }
                else {
                    //TODO: Implement calculating results if needed
                    (name, 0)
                }
            })
            .collect();

        Ok(results)
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
