use std::{env, str::FromStr, cmp::Ordering};
use core::fmt::Display;

use chrono::{ Local, NaiveDate };
use serde_json::{Value, Map};
use serenity::model::id::EmojiId;

use crate::database::WeekPicks;

pub fn get_short_name(name: &str) -> String {
    match name {
        "Arizona Cardinals"    => "ARI",
        "Atlanta Falcons"      => "ATL",
        "Baltimore Ravens"     => "BAL",
        "Buffalo Bills"        => "BUF",
        "Carolina Panthers"    => "CAR",
        "Chicago Bears"        => "CHI",
        "Cincinnati Bengals"   => "CIN",
        "Cleveland Browns"     => "CLE",
        "Dallas Cowboys"       => "DAL",
        "Denver Broncos"       => "DEN",
        "Detroit Lions"        => "DET",
        "Green Bay Packers"    => "GB",
        "Houston Texans"       => "HOU",
        "Indianapolis Colts"   => "IND",
        "Jacksonville Jaguars" => "JAX",
        "Kansas City Chiefs"   => "KC",
        "Los Angeles Rams"     => "LA",
        "Los Angeles Chargers" => "LAC",
        "Las Vegas Raiders"    => "LV",
        "Oakland Raiders"      => "LV",
        "Miami Dolphins"       => "MIA",
        "Minnesota Vikings"    => "MIN",
        "New England Patriots" => "NE",
        "New Orleans Saints"   => "NO",
        "New York Giants"      => "NYG",
        "New York Jets"        => "NYJ",
        "Philadelphia Eagles"  => "PHI",
        "Pittsburgh Steelers"  => "PIT",
        "Seattle Seahawks"     => "SEA",
        "San Francisco 49ers"  => "SF",
        "Tampa Bay Buccaneers" => "TB",
        "Tennessee Titans"     => "TEN",
        "Washington"           => "WAS",
        "Washington Commanders"=> "WAS",
        "Washington Redskins"  => "WAS",
        _                      => "N/A",
    }.to_owned()
}

pub fn get_team_emoji(team: &str) -> EmojiId {
    return EmojiId(match team {
        "ARI" => 1142671366424887367,
        "ATL" => 1142671368161341491,
        "BAL" => 1142671369008582697,
        "BUF" => 1142671369956507668,
        "CAR" => 1142671371260932197,
        "CHI" => 1142671373139968040,
        "CIN" => 1142671374515703868,
        "CLE" => 1142671375941783685,
        "DAL" => 1142671377736925234,
        "DEN" => 1142671664379875459,
        "DET" => 1142671665449410570,
        "GB"  => 1142671674727223507,
        "HOU" => 1142671676417523731,
        "IND" => 1142671380832338000,
        "JAX" => 1142671677990387722,
        "KC"  => 1142671679051546724,
        "LA"  => 1142671680410484888,
        "LAC" => 1142671682121773097,
        "LV"  => 1142671384263270410,
        "MIA" => 1142671683325526126,
        "MIN" => 1142671684395094086,
        "NE"  => 1142671686001512538,
        "NO"  => 1142671388507918356,
        "NYG" => 1142671779022770237,
        "NYJ" => 1142671392026923148,
        "PHI" => 1142671781107347606,
        "PIT" => 1142671688723603496,
        "SEA" => 1142671395256541225,
        "SF"  => 1142671782139134043,
        "TB"  => 1142671784433430570,
        "TEN" => 1142671692989218937,
        "WAS" => 1142671397987041281,
        _     => 1142674584508825681,
    });
}

pub struct Match {
    pub id_event: String,
    pub away_team: String,
    pub home_team: String,
    pub away_score: Option<u64>,
    pub home_score: Option<u64>,
    pub date: NaiveDate,
}

impl Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} - {}] {} - {:?} VS. {:?} - {}",
            self.id_event, self.date,
            self.away_team, self.away_score,
            self.home_score, self.home_team
        )
    }
}

pub async fn get_week(season: &u16, week: &i64) -> Option<impl Iterator<Item=Match>> {
    let partial_url = env::var("FOOTBALL_URL")
        .expect("![Football] Could not find 'FOOTBALL_URL' env var");
    let league = env::var("CONF_LEAGUE")
        .expect("![Football] Could not find 'CONF_LEAGUE' env var")
        .parse::<u16>()
        .expect("![Football] Could not parse 'CONF_LEAGUE' to int");

    let url = format!("{}?id={}&s={}&r={}", partial_url, league, season, week);

    let res = reqwest::get(url).await
        .expect("![Football] Could not get reply")
        .text().await
        .expect("![Football] Could not retrieve text from response");

    let json: Value = serde_json::from_str(res.as_str())
        .expect("![Football] Could not parse response");
    if let Value::Object(o) = json {
        let events = o.get("events").expect("![Football] Could not find key 'events'")
            .as_array().expect("![Football] Could not parse 'events' as an array")
            .to_owned();

        return Some(events.into_iter().map(move |m| {
            Match {
                id_event: m["idEvent"].as_str().unwrap().to_owned(),
                away_team: get_short_name(m["strAwayTeam"].as_str().unwrap()),
                home_team: get_short_name(m["strHomeTeam"].as_str().unwrap()),
                away_score: m["intAwayScore"].as_str().unwrap_or("").parse().ok(),
                home_score: m["intHomeScore"].as_str().unwrap_or("").parse().ok(),
                date: NaiveDate::from_str(m["dateEvent"].as_str().unwrap()).unwrap(),
            }
        }));
    } else {
        None
    }
}

pub struct PickResults {
    pub pickid: Option<i64>,
    pub name: String,
    pub score: u32,
    pub icons: String,
    pub cache: bool,
}

impl Display for PickResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {} - {} (should cache? {})",
            self.pickid, self.name, self.score, self.cache)
    }
}

pub async fn calc_results(week: &i64, matches: &[Match], picks: &[WeekPicks]) -> Vec<PickResults> {
    let now = Local::now().date_naive();
    let week_complete = matches.iter().all(|m| {
        match m.date.cmp(&now) {
            Ordering::Less => {
                true
            },
            Ordering::Equal | Ordering::Greater => {
                false
            },
        }
    });

    picks.iter()
        .map(|p| {
            let name = p.name.to_owned();

            let icons = if let Some(picks) = &p.picks {
                matches.iter().fold(String::new(), |mut acc, m| {
                    let choice = match picks.get(&m.id_event) {
                        Some(p) => p.as_str().unwrap(),
                        None => "NA",
                    };

                    acc.push_str(format!("<:{}:{}>", choice, get_team_emoji(choice)).as_str());
                    acc
                })
            }
            else {
                String::new()
            };

            if let Some(cached) = p.cached {
                PickResults {
                    pickid: p.pickid, name, score: cached, icons,
                    cache: false && week_complete && p.pickid.is_some()
                }
            }
            else {
                let (score, cache) = match &p.picks {
                    Some(poolerpicks) => (
                        calc_results_internal( &matches, &week, picks, &poolerpicks, p.poolerid),
                        true && week_complete && p.pickid.is_some()),
                    None => (0, false && week_complete && p.pickid.is_some()),
                };
                PickResults { pickid: p.pickid, name, score, icons, cache }
            }
        })
        .collect()
}

#[derive(Debug)]
enum MatchOutcome {
    Win,
    Loss,
    Tied,
}

fn calc_results_internal(
    matches: &[Match],
    week: &i64,
    poolpicks: &[WeekPicks],
    picks: &Map<String, Value>,
    poolerid: i64) -> u32 {

    let total = matches.iter().fold(0, |acc, m| {
        if let Some(pick) = picks.get(&m.id_event) {
            let pick = pick.as_str()
                .expect("[results] Could not get match pick as str");

            let unique = poolpicks.iter()
                .filter(|&pp| pp.poolerid != poolerid)
                .map(|pp| { 
                    match &pp.picks {
                        Some(p) => p.get(&m.id_event).unwrap().as_str().unwrap(),
                        None => "",
                    }
                })
                .all(|pp| pp != pick);

            let outcome = match m.away_score.cmp(&m.home_score) {
                Ordering::Less => if pick == m.away_team { MatchOutcome::Loss } else { MatchOutcome::Win },
                Ordering::Greater => if pick == m.away_team { MatchOutcome::Win } else { MatchOutcome::Loss },
                Ordering::Equal => MatchOutcome::Tied,
            };

            acc + get_score(&outcome, unique, &week)
        }
        else {
            acc
        }
    });
    total
}

fn get_score(outcome: &MatchOutcome, unique: bool, week: &i64) -> u32 {
    match outcome {
        MatchOutcome::Win => {
            match week {
                1..=18 => if unique { 3 } else { 2 },
                19 => if unique { 6 } else { 4 },
                20 => if unique { 9 } else { 6 },
                21 => if unique { 12 } else { 8 },
                22 => if unique { 15 } else { 10 },
                _ => 2,
            }
        },
        MatchOutcome::Loss => 0,
        MatchOutcome::Tied => 1,
    }
}
