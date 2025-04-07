use std::collections::HashMap;
use std::env;
use std::fmt::{ Display, Debug };

use anyhow::{anyhow, Result};
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
    pub week: i64,
    pub picks: Option<HashMap<String, String>>,
    pub featpick: Option<u32>,
    pub cached: Option<u32>,
}

type SeasonPicks = Vec<(i64, String, Vec<WeekPicks>)>;

pub enum PicksStatus {
    Primed(i64),
    Filled(String, Option<u32>),
}

impl Display for WeekPicks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(pickid: {:?}, poolerid: {})[{}] ({:?}) -> {:?} (feat: {:?})",
            self.pickid, self.poolerid,
            self.name,
            self.cached, self.picks, self.featpick)
    }
}

impl Debug for WeekPicks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(pickid: {:?}, poolerid: {})[{}] ({:?}) -> {:?} (feat: {:?})",
            self.pickid, self.poolerid,
            self.name,
            self.cached, self.picks, self.featpick)
    }
}

pub struct WeekFeature {
    pub season: i16,
    pub week: i64,
    pub feattype: i32,
    pub target: i32,
    pub matchid: String,
}

impl Display for WeekFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "For ({}, {}) - {} : [{}] value = {}",
            self.season, self.week,
            self.matchid,
            self.feattype, self.target)
    }
}

impl Debug for WeekFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "For ({}, {}) - {} : [{}] value = {}",
            self.season, self.week,
            self.matchid,
            self.feattype, self.target)
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
                SELECT pk.id as 'pickid', pl.id as 'poolerid', pl.name, pk.pickstring, pk.featurepick, pk.scorecache FROM poolers AS pl
                LEFT JOIN (
                    SELECT id, poolerid, pickstring, featurepick, scorecache FROM picks
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
                let picks = if let Some(pickmap) = picks { 
                    let m = pickmap.iter().fold(HashMap::new(), |mut map, (key, val)| {
                        map.insert(key.to_owned(), val.as_str().unwrap().to_owned());
                        map
                    });
                    Some(m)
                } else {
                    None
                };
                let featpick: Option<u32> = row.get("featurepick");
                let cached: Option<u32> = row.get("scorecache");

                WeekPicks { pickid, poolerid, name, week: *week, picks, featpick, cached }
            })
            .collect();
        
        Ok(results)
    }

    pub async fn fetch_pick(&self, season: &u16, week: &i64, poolerid: &i64) -> Result<WeekPicks> {
        let pickrow = sqlx::query("
                SELECT pk.id AS 'pickid', pl.id AS 'poolerid', pl.name, pk.pickstring, pk.featurepick, pk.scorecache FROM picks AS pk
                JOIN poolers AS pl ON pl.id = pk.poolerid
                WHERE season = ? AND week = ? AND poolerid = ?
                ")
            .bind(season)
            .bind(week)
            .bind(poolerid)
            .fetch_one(&self.pool).await?;

        Ok(WeekPicks {
            pickid: pickrow.get("pickid"),
            poolerid: pickrow.get("poolerid"),
            name: pickrow.get("name"),
            week: *week,
            picks: serde_json::from_str(pickrow.get("pickstring")).ok(),
            featpick: pickrow.get("featurepick"),
            cached: pickrow.get("scorecache")
        })
    }

    pub async fn fetch_season(&self, poolid: &i64, season: &u16) -> Result<(SeasonPicks, usize)> {
        let feat_ids: Vec<String> = sqlx::query("
            SELECT week, match FROM features
            SORT BY week
            ")
        .bind(season)
        .fetch_all(&self.pool).await.unwrap_or_else(|_| vec![])
        .iter().map(|r|
            r.get("match")
        ).collect();

        let season = sqlx::query("
                    SELECT pk.id as 'pickid', pl.id as 'poolerid', pl.name, pk.week, pk.scorecache, pk.pickstring, pk.featurepick FROM picks AS pk
                    LEFT JOIN (
                        SELECT id, name, poolid FROM poolers
                        WHERE poolid = ?
                    ) AS pl ON pk.poolerid = pl.id
                    WHERE season = ?
                    ORDER BY week, poolerid
                ")
            .bind(poolid)
            .bind(season)
            .fetch_all(&self.pool).await?
            .iter().map(|row| {
                let pickid: Option<i64> = row.get("pickid");
                let poolerid: i64 = row.get("poolerid");
                let name: String = row.get("name");
                let week: i64 = row.get("week");
                //TODO: There has to be a better way
                let picks: Option<Map<String, Value>> = serde_json::from_str(row.get("pickstring")).ok();
                let picks = if let Some(pickmap) = picks { 
                    let m = pickmap.iter().fold(HashMap::new(), |mut map, (key, val)| {
                        map.insert(key.to_owned(), val.as_str().unwrap().to_owned());
                        map
                    });
                    Some(m)
                } else {
                    None
                };
                let featpick: Option<u32> = row.get("featurepick");
                let cached: Option<u32> = row.get("scorecache");

                WeekPicks { pickid, poolerid, name, week, picks, featpick, cached }
            })
            .fold(SeasonPicks::new(), |mut acc, e| {
                if let Some(week) = acc.iter_mut().find(|a| a.0 == e.week) {
                    // Add pooler's picks to week's group
                    week.2.push(e);
                }
                else {
                    // Add new week group
                    if let Some(feat_id) = feat_ids.get((e.week - 1) as usize) {
                        acc.push( (e.week, feat_id.clone(), vec![e]) );
                    } else {
                        acc.push( (e.week, String::new(), vec![e]) );
                    }
                }
                acc
            });

        let week_count = season.len();
        Ok((season, week_count))
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
                SELECT id, pickstring, featurepick FROM picks
                WHERE poolerid = ? AND season = ? AND week = ?
                ")
            .bind(poolerid)
            .bind(season)
            .bind(week)
            .fetch_one(&self.pool)
            .await {
                Ok(row) => {
                    if let Some(pickstring) = row.get::<Option<String>, &str>("pickstring") {
                        let featpick = row.get::<Option<u32>, &str>("featurepick");
                        Ok(PicksStatus::Filled(pickstring, featpick))
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

    pub async fn fetch_poolerid(&self, discordid: &i64,) -> Result<i64> {
        let row = sqlx::query("
                SELECT p.id FROM poolers AS p
                JOIN users AS u
                ON u.id = p.userid
                WHERE discordid = ?
                ")
            .bind(discordid)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get(0))
    }

    pub async fn fetch_favteam(&self, discordid: &i64) -> Result<(String, String)> {
        let row = sqlx::query("
                SELECT p.name, p.favteam FROM users AS u
                JOIN poolers AS p
                ON p.userid = u.id
                WHERE discordid = ?
                ")
            .bind(discordid)
            .fetch_one(&self.pool)
            .await?;

        Ok((row.get(0), row.get(1)))
    }

    pub async fn update_favteam(&self, discordid: &i64, team: &str) -> Result<bool> {
        match sqlx::query("
                UPDATE poolers
                SET favteam = ?
                WHERE userid IN (
                    SELECT id from users
                    WHERE discordid = ?
                )
                ")
            .bind(team)
            .bind(discordid)
            .execute(&self.pool)
            .await {
                Ok(r) => {
                    println!("[DB] Successful favorite team updated: rows affected {}", r.rows_affected());
                    Ok(true)
                },
                Err(e) => {
                    println!("[DB] Could not update score cache: {}", e);
                    Ok(false)
                }
        }
    }

    pub async fn fetch_feature(&self, season: u16, week: i64) -> Result<WeekFeature> {
        match sqlx::query("
            SELECT ft.season, ft.week, ft.type, ft.target, ft.match FROM features AS ft
            WHERE season = ? AND week = ?;
        ")
        .bind(season)
        .bind(week)
        .fetch_one(&self.pool).await
        {
            Ok(r) => {
                Ok(WeekFeature {
                    season: r.get("season"),
                    week: r.get("week"),
                    feattype: r.get("type"),
                    target: r.get("target"),
                    matchid: r.get("match"),
                })
            },
            Err(e) => {
                Err(anyhow!(e))
            }
        }
    }

    pub async fn set_feature(&self, season: u16, week: i64, target: i64, matchid: String) -> Result<bool> {
        //TODO: Select before to know wether to INSERT INTO or to UPDATE
        match sqlx::query("
                INSERT INTO features (season, week, type, target, match)
                VALUES (?, ?, ?, ?, ?);
        ")
        .bind(season)
        .bind(week)
        .bind(0)
        .bind(target)
        .bind(matchid)
        .execute(&self.pool)
        .await {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => {
                println!("[DB] Could not insert new feature: {}", e);
                Ok(false)
            }
        }
    }
}
