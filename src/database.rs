use std::fmt::Display;

use anyhow::Result;
use sqlx::{ Pool, Sqlite };
use sqlx::Row;

pub struct PoolerResult {
    pub pooler_name: String,
    pub match_ids: Vec<String>,
    pub results: Vec<i8>,
}

impl Display for PoolerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("{}\n", self.pooler_name);

        self.match_ids.iter().enumerate().for_each(|(idx, mid)| {
            out.push_str(format!("\t{}: {}\n", mid, self.results[idx]).as_str());
        });

        writeln!(f, "{out}")
    }
}

pub async fn fetch_results(db: &Pool<Sqlite>, _week: u64) -> Result<()> {
    let users = sqlx::query("SELECT id, email, discordid FROM Users")
        .fetch_all(db)
        .await?;

    for row in users {
        let email = row.get::<String, &str>("email");
        println!("{}", email);
    }

    Ok(())
}
