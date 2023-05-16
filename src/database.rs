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

    pub async fn fetch_results(&self, week: u64) -> Result<()> {
        println!("[DB] Getting results for week {week}");
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
}
