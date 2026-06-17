use std::collections::{HashMap, BTreeMap};
use std::env;
use std::fmt::{ Display, Debug };

use anyhow::Result;
use sqlx::{ Pool, QueryBuilder, Row, Sqlite };
use sqlx::sqlite::{ SqlitePool, SqliteRow };

pub struct DB {
    pool: Pool<Sqlite>,
}

pub struct WeekPicks {
    pub pickid: Option<i64>,
    pub poolerid: i64,
    pub name: String,
    pub week: i64,
    pub picks: Option<HashMap<String, String>>,
    pub counts: Option<HashMap<String, i32>>,
    pub featpick: Option<u32>,
    pub cached: Option<u32>,
    pub featcached: Option<u32>,
}

type SeasonPicks = Vec<(i64, Option<WeekFeature>, Vec<WeekPicks>)>;

#[derive(Debug, Default)]
pub struct CapsulePicks {
    pub season: u16,
    pub poolerid: i64,
    pub name: String,

    pub nfc_wins: [String; 4],
    pub nfc_wins_counts: [i32; 4],
    pub nfc_wildcards: [String; 3],
    pub nfc_wild_counts: [i32; 3],

    pub afc_wins: [String; 4],
    pub afc_wins_counts: [i32; 4],
    pub afc_wildcards: [String; 3],
    pub afc_wild_counts: [i32; 3],
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
        let rows = sqlx::query("
                SELECT pl.id AS 'poolerid', pl.name,
                   pk.id AS 'pickid', pk.featurepick, pk.scorecache, pk.featcache,
                   mp.matchid, mp.team,
                   COUNT(*) OVER (PARTITION BY mp.matchid, mp.team) AS 'pick_count'
                FROM poolers AS pl
                LEFT JOIN picks AS pk ON pk.poolerid = pl.id AND pk.season = ? AND pk.week = ?
                LEFT JOIN match_picks AS mp ON mp.pickid = pk.id
                WHERE pl.poolid = ?
                ORDER BY pl.id
                ")
            .bind(season)
            .bind(week)
            .bind(poolid)
            .fetch_all(&self.pool).await?;

        let mut by_pooler: BTreeMap<i64, WeekPicks> = BTreeMap::new();
        for row in &rows {
            let poolerid: i64 = row.get("poolerid");
            let entry = by_pooler.entry(poolerid).or_insert_with(|| {
                let pickid: Option<i64> = row.get("pickid");
                WeekPicks {
                    pickid,
                    poolerid,
                    name: row.get("name"),
                    week: *week,
                    picks: pickid.map(|_| HashMap::new()),
                    counts: pickid.map(|_| HashMap::new()),
                    featpick: row.get("featurepick"),
                    cached: row.get("scorecache"),
                    featcached: row.get("featcache"),
                }
            });

            if let Some(matchid) = row.get::<Option<String>, _>("matchid") {
                if let Some(map) = entry.picks.as_mut() {
                    map.insert(matchid.clone(), row.get("team"));
                }
                if let Some(count) = entry.counts.as_mut() {
                    count.insert(matchid, row.get("pick_count"));
                }
            }
        }

        Ok(by_pooler.into_values().collect())
    }

    pub async fn fetch_pick(&self, season: &u16, week: &i64, poolerid: &i64) -> Result<WeekPicks> {
        let rows = sqlx::query("
                SELECT pk.id AS 'pickid', pl.id AS 'poolerid', pl.name,
                   pk.featurepick, pk.scorecache, pk.featcache,
                   mp.matchid, mp.team,
                   COUNT(*) OVER (PARTITION BY mp.matchid, mp.team) AS 'pick_count'
                FROM picks AS pk
                JOIN poolers AS pl ON pl.id = pk.poolerid
                LEFT JOIN match_picks AS mp ON mp.pickid = pk.id
                WHERE pk.season = ? AND pk.week = ? AND pk.poolerid = ?
                ")
            .bind(season)
            .bind(week)
            .bind(poolerid)
            .fetch_all(&self.pool).await?;

        // No parent row -> not submitted yet; the caller treats this as "issue a token"
        let first = match rows.first() {
            Some(row) => row,
            None => return Err(anyhow::anyhow!("no picks for pooler {poolerid} (season {season}, week {week})")),
        };

        let mut map = HashMap::<String, String>::new();
        let mut map_counts = HashMap::<String, i32>::new();
        for row in &rows {
            if let Some(matchid) = row.get::<Option<String>, _>("matchid") {
                map.insert(matchid.clone(), row.get("team"));
                map_counts.insert(matchid, row.get("pick_count"));
            }
        }
        let picks = if map.is_empty() { None } else { Some(map) };
        let counts = if map_counts.is_empty() { None } else { Some(map_counts) };

        Ok(WeekPicks {
            pickid: first.get("pickid"),
            poolerid: first.get("poolerid"),
            name: first.get("name"),
            week: *week,
            picks,
            counts,
            featpick: first.get("featurepick"),
            cached: first.get("scorecache"),
            featcached: first.get("featcache")
        })
    }

    pub async fn fetch_season(&self, poolid: &i64, season: &u16) -> Result<(SeasonPicks, usize)> {
        let mut feats: HashMap<_, _> = sqlx::query("
            SELECT season, week, type, target, match FROM features
            ORDER BY week
            ")
        .bind(season)
        .fetch_all(&self.pool).await.unwrap_or_else(|_| vec![])
        .into_iter().map(|r| {
            let week: i64 = r.get("week");
            let feat = WeekFeature {
                season: r.get("season"),
                week,
                feattype: r.get("type"),
                target: r.get("target"),
                matchid: r.get("match"),
            };
            (week, feat)
        })
        .collect();

        let season_rows = sqlx::query("
                    SELECT pk.id as 'pickid', pl.id as 'poolerid', pl.name,
                        pk.week, pk.scorecache, pk.featurepick, pk.featcache,
                        mp.matchid, mp.team,
                        COUNT(*) OVER (PARTITION BY mp.matchid, mp.team) AS 'pick_count'
                    FROM picks AS pk
                    JOIN poolers AS pl ON pl.id = pk.poolerid AND pl.poolid = ?
                    LEFT JOIN match_picks AS mp ON mp.pickid = pk.id
                    WHERE pk.season = ?
                    ORDER BY pk.week, pl.id
                ")
            .bind(poolid)
            .bind(season)
            .fetch_all(&self.pool).await?;

        let mut by_key: BTreeMap<(i64, i64), WeekPicks> = BTreeMap::new();
        for row in &season_rows {
            let week: i64 = row.get("week");
            let poolerid: i64 = row.get("poolerid");
            let entry = by_key.entry((week, poolerid)).or_insert_with(|| {
                let pickid: Option<i64> = row.get("pickid");
                WeekPicks {
                    pickid,
                    poolerid,
                    name: row.get("name"),
                    week,
                    picks: pickid.map(|_| HashMap::new()),
                    counts: pickid.map(|_| HashMap::new()),
                    featpick: row.get("featurepick"),
                    cached: row.get("scorecache"),
                    featcached: row.get("featcache"),
                }
            });

            if let Some(matchid) = row.get::<Option<String>, _>("matchid") {
                if let Some(map) = entry.picks.as_mut() {
                    map.insert(matchid.clone(), row.get("team"));
                }
                if let Some(count) = entry.counts.as_mut() {
                    count.insert(matchid, row.get("pick_count"));
                }
            }
        }

        let season = by_key.into_values().fold(SeasonPicks::new(), |mut acc, e| {
            if let Some(week) = acc.iter_mut().find(|a| a.0 == e.week) {
                week.2.push(e);
            }
            else {
                if let Some(feat) = feats.remove(&e.week) {
                    acc.push( (e.week, Some(feat), vec![e]) );
                } else {
                    acc.push( (e.week, None, vec![e]) );
                }
            }
            acc
        });

        let week_count = season.len();
        Ok((season, week_count))
    }

    pub async fn issue_pick_token(&self, season: u16, week: i64, poolerid: i64) -> Result<i64> {
        let row = sqlx::query("
                INSERT INTO pick_tokens (poolerid, token, season, week)
                VALUES (?, random() & 0x7fffffffffffffff, ?, ?)
                ON CONFLICT(poolerid) DO UPDATE SET
                    token  = excluded.token,
                    season = excluded.season,
                    week   = excluded.week
                RETURNING token;
                ")
            .bind(poolerid)
            .bind(season)
            .bind(week)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("token"))
    }

    pub async fn cache_results(&self, pickid: &i64, score: &u32, featscore: &u32) -> Result<bool> {
        match sqlx::query("
                UPDATE picks
                SET scorecache = ?, featcache = ?
                WHERE id = ?
                ")
            .bind(score)
            .bind(featscore)
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
        let row = sqlx::query("
            SELECT ft.season, ft.week, ft.type, ft.target, ft.match FROM features AS ft
            WHERE season = ? AND week = ?;
        ")
        .bind(season)
        .bind(week)
        .fetch_one(&self.pool).await?;

        Ok(WeekFeature {
            season: row.get("season"),
            week: row.get("week"),
            feattype: row.get("type"),
            target: row.get("target"),
            matchid: row.get("match"),
        })
    }

    pub async fn set_feature(&self, season: u16, week: i64, target: i64, matchid: &String) -> Result<bool> {
        match self.fetch_feature(season, week).await {
            Ok(_) => {
                // feature for season/week found, UPDATE existing
                let outcome = sqlx::query("
                        UPDATE features
                        SET type = ?, target = ?, match = ?
                        WHERE season = ? AND week = ?;
                ")
                .bind(0)
                .bind(target)
                .bind(matchid)
                .bind(season)
                .bind(week)
                .execute(&self.pool)
                .await;

                //TODO: Probably need better error handling here
                match outcome {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => {
                // Could not find feature for season/week, INSERT new
                let outcome = sqlx::query("
                        INSERT INTO features (season, week, type, target, match)
                        VALUES (?, ?, ?, ?, ?);
                ")
                .bind(season)
                .bind(week)
                .bind(0)
                .bind(target)
                .bind(matchid)
                .execute(&self.pool)
                .await;

                //TODO: Probably need better error handling here
                match outcome {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    pub async fn fetch_pooler_capsule(&self, discordid: &i64, season: u16) -> Result<Option<CapsulePicks>> {
        let prow = sqlx::query("
                SELECT p.id, p.name FROM users AS u
                JOIN poolers AS p
                ON u.id = p.userid
                WHERE u.discordid = ?
                ")
            .bind(discordid)
            .fetch_one(&self.pool)
            .await?;
        let poolerid: i64 = prow.get("id");
        let name: String = prow.get("name");

        let rows: Vec<SqliteRow> = sqlx::query("
                SELECT type, conference, division, slot, team,
                       COUNT(*) OVER (PARTITION BY season, type, conference, division, team) AS pick_count
                FROM capsules
                WHERE poolerid = ? AND season = ?
                ")
            .bind(poolerid)
            .bind(season)
            .fetch_all(&self.pool)
            .await?;

        match rows.len() {
            0 => Ok(None),
            14 => {
                let mut capsule = CapsulePicks { season, poolerid, name, ..Default::default() };
                for row in &rows { populate_capsule(&mut capsule, row); }
                Ok(Some(capsule))
            },
            n => panic!("[DB] pooler {poolerid} has {n} capsule rows for season {season} (expected 0 or 14)"),
        }
    }

    pub async fn fetch_capsule(&self, season: &u16, poolid: &i64) -> Result<HashMap<i64, CapsulePicks>> {
        let poolerids: Vec<i64>= sqlx::query("
                SELECT id FROM poolers
                WHERE poolid = ?
                ")
            .bind(poolid)
            .fetch_all(&self.pool)
            .await?
            .iter().map(|row| row.get("id"))
            .collect();

        let mut qb = QueryBuilder::new("
                SELECT c.season, c.poolerid, p.name, c.type, c.conference,
                       c.division, c.slot, c.team,
                       COUNT(*) OVER (PARTITION BY c.season, c.type, c.conference, c.division, c.team) AS pick_count
                FROM capsules AS c
                JOIN poolers AS p ON p.id = c.poolerid
                WHERE c.season = 
                ");
        qb.push_bind(*season);
        qb.push(" AND c.poolerid IN (");
        let mut list = qb.separated(",");
        for pid in poolerids {
            list.push_bind(pid);
        }
        list.push_unseparated(") ORDER BY c.season, c.poolerid, c.conference, c.type, c.division, c.slot");

        let rows: Vec<_> = qb.build().fetch_all(&self.pool).await?;
        let mut capsules = HashMap::<i64, CapsulePicks>::new();

        for row in &rows {
            let poolerid: i64 = row.get("poolerid");
            let name: String = row.get("name");

            let capsule = capsules.entry(poolerid).or_insert_with(|| { CapsulePicks { season: *season, poolerid, name, ..Default::default() } });
            populate_capsule(capsule, row);
        }

        Ok(capsules)
    }
}

fn populate_capsule(capsule: &mut CapsulePicks, row: &SqliteRow) {
    let conf: i32 = row.get("conference");
    let t: i32 = row.get("type");
    let team: String = row.get("team");
    let count: i32 = row.get("pick_count");

    match (conf, t) {
        (0, 0) => {
            let i = row.get::<i32, _>("division") as usize;
            capsule.nfc_wins[i] = team;
            capsule.nfc_wins_counts[i] = count;
        },
        (0, 1) => {
            let i = row.get::<i32, _>("slot") as usize;
            capsule.nfc_wildcards[i] = team;
            capsule.nfc_wild_counts[i] = count;
        },
        (1, 0) => {
            let i = row.get::<i32, _>("division") as usize;
            capsule.afc_wins[i] = team;
            capsule.afc_wins_counts[i] = count;
        },
        (1, 1) => {
            let i = row.get::<i32, _>("slot") as usize;
            capsule.afc_wildcards[i] = team;
            capsule.afc_wild_counts[i] = count;
        },
        _ => unreachable!("[DB] Could not parse capsule row poolerid = {}; conference = {conf}; type = {t}", capsule.poolerid),
    };
}
