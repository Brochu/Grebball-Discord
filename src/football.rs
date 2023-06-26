use std::{env, str::FromStr, cmp::Ordering};
use core::fmt::Display;

use chrono::{ Local, NaiveDate };
use serde_json::{Value, Map};
use serenity::model::id::EmojiId;

use crate::database::{ DB, WeekPicks };

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
        "ARI" => 1101771924398424115,
        "ATL" => 1101771925853847582,
        "BAL" => 1101771926738829332,
        "BUF" => 1101771927724490843,
        "CAR" => 1101771928542400512,
        "CHI" => 1101771930069127188,
        "CIN" => 1101771930677301270,
        "CLE" => 1101771931868463125,
        "DAL" => 1101772137217400892,
        "DEN" => 1101772138664443914,
        "DET" => 1101772139545235456,
        "GB"  => 1101772244583206963,
        "HOU" => 1101771936352186388,
        "IND" => 1101772246214787072,
        "JAX" => 1101772247200440321,
        "KC"  => 1101771938868768840,
        "LA"  => 1101772371418947584,
        "LAC" => 1101772373029552189,
        "LV"  => 1101772373964882000,
        "MIA" => 1101771943377633361,
        "MIN" => 1101772510556602398,
        "NE"  => 1101772512121073664,
        "NO"  => 1101772512922181662,
        "NYG" => 1101772513782026381,
        "NYJ" => 1101772514662817843,
        "PHI" => 1101771950663155802,
        "PIT" => 1101772694015459408,
        "SEA" => 1101771953745965126,
        "SF"  => 1101772694875279521,
        "TB"  => 1101772696716574780,
        "TEN" => 1101772697580605522,
        "WAS" => 1101771957831221338,
        _     => 0,
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
    let league = env::var("CONF_LEAGUE")
        .expect("![Week] Could not find 'CONF_LEAGUE' env var")
        .parse::<u16>()
        .expect("![Week] Could not parse 'CONF_LEAGUE' to int");

    let url = format!("https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id={}&s={}&r={}",
        league, season, week);

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
                away_score: m["intAwayScore"].as_str().unwrap().parse().ok(),
                home_score: m["intHomeScore"].as_str().unwrap().parse().ok(),
                date: NaiveDate::from_str(m["dateEvent"].as_str().unwrap()).unwrap(),
            }
        }));
    } else {
        None
    }
}

//TODO: Make a version to check for a full season for a given poolid
pub async fn calc_results(season: &u16, week: &i64, picks: &[WeekPicks], db: &DB) -> String {
    let matches: Vec<Match> = get_week(&season, &week).await
        .expect("[results] Could not fetch week data")
        .collect();
    let results: Vec<(i64, String, u32, bool, String)> = picks.iter()
        .map(|p| {
            let name = p.name.to_owned();
            let (score, should_cache) = if let Some(cached) = p.cached {
                (cached, false)
            }
            else {
                if let Some(poolerpicks) = &p.picks {
                    (calc_results_internal(&matches, &week, &picks, &poolerpicks, p.poolerid), true)
                }
                else {
                    (0, false)
                }
            };

            //TODO: Check if this works with NULL picks
            let icons = if let Some(poolerpicks) = &p.picks {
                matches.iter().fold(String::new(), |mut str, m| {
                    let choice = poolerpicks.get(&m.id_event)
                        .unwrap().as_str()
                        .unwrap();
                    str.push_str(format!("<:{}:{}>", choice, get_team_emoji(choice)).as_str());

                    str.push(' ');
                    str
                })
            }
            else {
                String::new()
            };

            (p.pickid, name, score, should_cache, icons)
        })
        .collect();

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

    let mut message = String::new();
    for (pickid, name, score, should_cache, icons) in results.iter() {

        if *should_cache && week_complete {
            if let Err(e) = db.cache_results(pickid, &score).await {
                println!("[results] Error while trying to cache score: {e}")
            }
        }

        let width = 10 - name.len();
        message.push_str(format!("`{name}{}`->", " ".repeat(width)).as_str());

        message.push_str(format!("{icons} |").as_str());
        message.push_str(format!(" {score}\n").as_str());
    }
    message
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
