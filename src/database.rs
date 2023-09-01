use std::env;
use std::fmt::Display;

use anyhow::Result;
use serde_json::{Value, Map};
use sqlx::{ Pool, Sqlite, Row };
use sqlx::sqlite::SqlitePool;

pub struct DB {
    pool: Pool<Sqlite>,
}

pub struct WeekPicks {
    pub pickid: Option<i64>,
    pub poolerid: i64,
    pub name: String,
    pub picks: Option<Map<String, Value>>,
    pub cached: Option<u32>,
}

pub enum PicksStatus {
    Primed(i64),
    Filled(String),
}

impl Display for WeekPicks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(pickid: {:?}, poolerid: {})[{}] ({:?}) -> {:?}",
            self.pickid, self.poolerid,
            self.name,
            self.cached, self.picks)
    }
}

impl DB {
    pub async fn new() -> DB {
        let db_url = env::var("DATABASE_URL")
            .expect("![MAIN] Cannot find 'DATABASE_URL' in env");

        DB { pool: SqlitePool::connect(db_url.as_str()).await.unwrap() }
    }

    pub async fn find_week(&self, poolid: &i64, season: &u16) -> Result<i64> {
        let week = sqlx::query("
                SELECT max(pk.week) as maxweek FROM picks as pk
                LEFT JOIN poolers as pl ON pk.poolerid = pl.id
                WHERE pl.poolid = ? AND season = ?
                GROUP BY pk.poolerid
                ")
            .bind(poolid)
            .bind(season)
            .fetch_all(&self.pool).await?
        .iter().fold( 0, |week, row| std::cmp::max(week, row.get("maxweek")) );

        Ok(week)
    }

    pub async fn fetch_picks(&self, poolid: &i64, season: &u16, week: &i64) -> Result<Vec<WeekPicks>> {
        let results: Vec<WeekPicks> = sqlx::query("
                SELECT pk.id as 'pickid', pl.id as 'poolerid', pl.name, pk.pickstring, pk.scorecache FROM poolers AS pl
                LEFT JOIN (
                    SELECT id, poolerid, pickstring, scorecache FROM picks
                    WHERE season = ? AND week = ?
                ) AS pk ON pk.poolerid = pl.id
                WHERE pl.poolid = ?
                ")
            .bind(season)
            .bind(week)
            .bind(poolid)
            .fetch_all(&self.pool).await?
            .iter().map(|row| {
                let pickid: Option<i64> = row.get("pickid");
                let poolerid: i64 = row.get("poolerid");
                let name: String = row.get("name");
                let picks: Option<Map<String, Value>> = serde_json::from_str(row.get("pickstring")).ok();
                let cached: Option<u32> = row.get("scorecache");

                WeekPicks { pickid, poolerid, name, picks, cached }
            })
            .collect();
        
        Ok(results)
    }

    pub async fn prime_picks(&self, discordid: &i64, season: &u16, week: &i64) -> Result<PicksStatus> {
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
                    if let Some(pickstring) = row.get::<Option<String>, &str>("pickstring") {
                        Ok(PicksStatus::Filled(pickstring))
                    }
                    else {
                        // Picks are already primed and not filled
                        let id: i64 = row.get("id");
                        Ok(PicksStatus::Primed(id))
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
                    Ok(PicksStatus::Primed(id))
                }
        }
    }

    pub async fn cache_results(&self, pickid: &i64, score: &u32) -> Result<bool> {
        match sqlx::query("
                UPDATE picks
                SET scorecache = ?
                WHERE id = ?
                ")
            .bind(score)
            .bind(pickid)
            .execute(&self.pool)
            .await {
                Ok(r) => {
                    println!("[DB] Successful score cache updated: rows affected {}", r.rows_affected());
                    Ok(true)
                },
                Err(e) => {
                    println!("[DB] Could not update score cache: {}", e);
                    Ok(false)
                }
        }
    }
}
