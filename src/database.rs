use std::fmt::Display;

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

pub async fn fetch_results(week: u64) -> Option<impl Iterator<Item=PoolerResult>> {
    //let matches = football::get_week(week);
    println!("For week: {}", week);

    let temp = vec![ PoolerResult {
        pooler_name: "".to_string(),
        match_ids: vec![],
        results: vec![]
    }];

    Some(temp.into_iter())
}
