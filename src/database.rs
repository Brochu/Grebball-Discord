use std::env;

use anyhow::Result;

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

        let users = sqlx::query("SELECT id, email, discordid FROM Users")
            .fetch_all(&self.pool)
            .await?;

        for row in users {
            let email = row.get::<String, &str>("email");
            println!("{}", email);
        }

        Ok(())
    }

    pub async fn prime_picks(&self, discordid: &i64, week: &i64) -> Result<i64> {
        //TODO: Need to check if picks entry exists already first
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
