use std::{env, cmp::Ordering, collections::HashMap};
use core::fmt::{Display, Debug};

use chrono::{ Utc, DateTime };
use serde::{Deserialize, Serialize};
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
        "Los Angeles Rams"     => "LAR",
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
        "Washington"           => "WSH",
        "Washington Commanders"=> "WSH",
        "Washington Redskins"  => "WSH",
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
        "LAR" => 1142671680410484888,
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
        "WSH" => 1142671397987041281,
        _     => 1142674584508825681,
    });
}

pub fn get_team_id(team: &str) -> i64 {
    match team {
        "ARI" => 22,
        "ATL" => 1,
        "BAL" => 33,
        "BUF" => 2,
        "CAR" => 29,
        "CHI" => 3,
        "CIN" => 4,
        "CLE" => 5,
        "DAL" => 6,
        "DEN" => 7,
        "DET" => 8,
        "GB"  => 9,
        "HOU" => 34,
        "IND" => 11,
        "JAX" => 30,
        "KC"  => 12,
        "LA"  => 14,
        "LAR" => 14,
        "LAC" => 24,
        "LV"  => 13,
        "MIA" => 15,
        "MIN" => 16,
        "NE"  => 17,
        "NO"  => 18,
        "NYG" => 19,
        "NYJ" => 20,
        "PHI" => 21,
        "PIT" => 23,
        "SEA" => 26,
        "SF"  => 25,
        "TB"  => 27,
        "TEN" => 10,
        "WAS" => 28,
        "WSH" => 28,
        _     => -1,
    }
}

#[derive(Clone)]
pub struct Match {
    pub id_event: String,
    pub away_team: String,
    pub home_team: String,
    pub away_score: Option<u64>,
    pub home_score: Option<u64>,
    pub date: DateTime<Utc>,
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

impl Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} - {}] {} - {:?} VS. {:?} - {}",
            self.id_event, self.date,
            self.away_team, self.away_score,
            self.home_score, self.home_team
        )
    }
}

pub async fn get_week(season: &u16, week: &i64) -> Option<impl Iterator<Item=Match>> {
    let data_url = env::var("DATA_URL")
        .expect("![Football] Could not find 'DATA_URL' env var");

    let (w, sw) = if *week == 19 { (160, 1) }
    else if *week == 20 { (125, 2) }
    else if *week == 21 { (150, 3) }
    else if *week == 22 { (200, 5) }
    else { (*week, *week) };
    let stype = if w < 100 { 2 } else { 3 };

    let scoreurl = format!("{}?dates={}&seasontype={}&week={}", data_url, season, stype, sw);
    let scoreres = reqwest::get(scoreurl).await
        .expect("![Football] Could not get reply")
        .text().await
        .expect("![Football] Could not retrieve text from response");
    let schedule: ESPNSchedule = serde_json::from_str(&scoreres)
        .expect("![Football] Could not parse response");

    let matches = schedule.events.into_iter().map(move |e| {
        let away_team = &e.comp[0].teams[1];
        let home_team = &e.comp[0].teams[0];
        let match_date = e.comp[0].date.replace("Z", ":00Z");
        Match {
            id_event: e.id,
            away_team: away_team.team.abbreviation.to_owned(),
            home_team: home_team.team.abbreviation.to_owned(),
            away_score: away_team.score.parse::<u64>().ok(),
            home_score: home_team.score.parse::<u64>().ok(),
            date: DateTime::parse_from_rfc3339(match_date.as_str())
                .map(|dt| dt.with_timezone(&Utc))
                .expect("![Football] Could not parse event's date")
        }
    });
    return Some(matches);
    /*
    if let Value::Object(so) = scoreres {
        let events = o.get("events").expect("![Football] Could not find key 'events'")
            .as_array().expect("![Football] Could not parse 'events' as an array")
            .to_owned();
        let scoreevents = so.get("events").expect("![Football] Could not find key 'events'")
            .as_array().expect("![Football] Could not parse 'events' as an array")
            .to_owned();

        return Some(events.into_iter().map(move |m| {
            let awayname = m["strAwayTeam"].as_str().unwrap();
            let homename = m["strHomeTeam"].as_str().unwrap();
            let comp = scoreevents.iter()
                .find(|&s| {
                    let name = s.as_object().unwrap()["name"].as_str().unwrap();
                    name.contains(awayname) && name.contains(homename)
                })
                .map(|r| {
                    r
                        .as_object().unwrap()["competitions"].as_array().unwrap()[0]
                        .as_object().unwrap()["competitors"].as_array().unwrap()
                })
                .unwrap();
            let (mut home_score, mut away_score) = (
                comp[0].as_object().unwrap()["score"].as_str().unwrap_or("").parse::<u64>().ok(),
                comp[1].as_object().unwrap()["score"].as_str().unwrap_or("").parse::<u64>().ok()
            );
            if home_score == Some(0) && away_score == Some(0) {
                (home_score, away_score) = (None, None);
            }

            Match {
                id_event: m["idEvent"].as_str().unwrap().to_owned(),
                away_team: get_short_name(m["strAwayTeam"].as_str().unwrap()),
                home_team: get_short_name(m["strHomeTeam"].as_str().unwrap()),
                away_score,
                home_score,
                date: NaiveDate::from_str(m["dateEvent"].as_str().unwrap()).unwrap(),
            }
        }));
    } else {
        None
    }
    */
}

#[derive(Serialize, Deserialize, Debug)]
struct ESPNSchedule {
    events: Vec<ESPNEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESPNEvent {
    id: String,
    week: ESPNWeek,
    #[serde(rename="competitions")]
    comp: Vec<ESPNCompetition>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESPNWeek {
    number: i8,
    //text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESPNCompetition {
    id: String,
    date: String,
    #[serde(rename="competitors")]
    teams: Vec<ESPNCompetitor>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESPNCompetitor {
    team: ESPNTeam,
    #[serde(default)]
    score: String,
}

/*
#[derive(Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
struct ESPNScore {
    value: f32,
    displayValue: String,
}
*/

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct ESPNTeam {
    abbreviation: String,
    displayName: String,
}

pub async fn get_schedule(season: &u16, teamid: &i64) -> Vec<Option<Match>> {
    let partial_url = env::var("BLAME_URL")
        .expect("![Football] Could not find 'BLAME_URL' env var");

    let url = format!("{}/{}/schedule?season={}", partial_url, teamid, season);
    let res = reqwest::get(url).await
        .expect("![Football] Could not get reply")
        .text().await
        .expect("![Football] Could not retrieve text from response");
    let schedule: ESPNSchedule = serde_json::from_str(&res).expect("![Football] Could not parse response");

    let matches: Vec<_> = schedule.events.iter().map(|e| {
        let hteam = &e.comp[0].teams[0];
        let ateam = &e.comp[0].teams[1];
        let match_date = e.comp[0].date.replace("Z", ":00Z");

        let (mut away_score, mut home_score) = (
            ateam.score.parse::<u64>().ok(),
            hteam.score.parse::<u64>().ok(),
        );
        if ateam.score.is_empty() && hteam.score.is_empty() {
            (away_score, home_score) = (None, None);
        }

        //TODO: Find a way to avoid all the clones
        (e.week.number, Match {
            id_event: e.comp[0].id.clone(),
            away_team: ateam.team.abbreviation.clone(), home_team: hteam.team.abbreviation.clone(),
            away_score, home_score,
            date: DateTime::parse_from_rfc3339(match_date.as_str())
                .map(|dt| dt.with_timezone(&Utc))
                .expect("![Football] Could not parse event's date"),
        })
    }).collect();

    let mut result = Vec::<Option<Match>>::new();
    result.resize(18, None);
    for (idx, m) in matches {
        result[(idx - 1) as usize] = Some(m);
    }

    return result;
}

/*
pub async fn test_get_week(season: &u16, week: &i64) -> Option<impl Iterator<Item=Match>> {
    let data_url = env::var("DATA_URL")
        .expect("![Football] Could not find 'DATA_URL' env var");

    let w = if *week == 19 { 160 }
    else if *week == 20 { 125 }
    else if *week == 21 { 150 }
    else if *week == 22 { 200 }
    else { *week };
    let stype = if w < 100 { 2 } else { 3 };

    let newurl = format!("{}?dates={}&seasontype={}&week={}", data_url, season, stype, w);
    let res = reqwest::get(newurl).await
        .expect("![Football] Could not get reply")
        .text().await
        .expect("![Football] Could not retrieve text from response");
    let json: Value = serde_json::from_str(res.as_str())
        .expect("![Football] Could not parse response");

    if let Value::Object(o) = json {
        let events = o.get("events").expect("![Football] Could not find key 'events'")
            .as_array().expect("![Football] Could not parse 'events' as an array")
            .to_owned();

        Some(events.into_iter().map(move |m| {
            let comp = m["competitions"]
                .as_array().unwrap()[0]
                .as_object().unwrap()["competitors"]
                .as_array().unwrap();
            let away = comp[1].as_object().unwrap()["team"]
                .as_object().unwrap()["abbreviation"]
                .as_str();
            let home = comp[0].as_object().unwrap()["team"]
                .as_object().unwrap()["abbreviation"]
                .as_str();

            let d = m["date"].as_str()
                .map(|s| s.replace("Z", ":00Z"))
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s).unwrap())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap();

            Match {
                id_event: m["id"].as_str().unwrap().to_owned(),
                away_team: away.unwrap().to_owned(),
                home_team: home.unwrap().to_owned(),
                away_score: comp[1].as_object().unwrap()["score"].as_str().unwrap_or("").parse().ok(),
                home_score: comp[0].as_object().unwrap()["score"].as_str().unwrap_or("").parse().ok(),
                date: d.date_naive() //TODO: Work to include time here later on
            }
        }))
    } else {
        None
    }
}
*/

pub fn calc_blame(
    _week: &i64,
    _matches: &[Match],
    _picks: &[WeekPicks],
    _poolerid: &i64,
    _team: &str) -> i64 {

    return 0;
}

pub struct PickResults {
    pub pickid: Option<i64>,
    pub poolerid: i64,
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
    let now = Utc::now();
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

    let mut output: Vec<PickResults> = picks.iter()
        .map(|p| {
            let name = p.name.to_owned();

            let icons = if let Some(picks) = &p.picks {
                matches.iter().fold(String::new(), |mut acc, m| {
                    let choice = match picks.get(&m.id_event) {
                        Some(p) => p.as_str(),
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
                    pickid: p.pickid, poolerid: p.poolerid, name, score: cached, icons,
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
                PickResults { pickid: p.pickid, poolerid: p.poolerid, name, score, icons, cache }
            }
        })
        .collect();

    output.sort_unstable_by(|l, r| { r.score.cmp(&l.score) });
    output
}

#[derive(Debug)]
enum MatchOutcome {
    Win,
    Loss,
    Tied,
    NotPlayed,
}

fn calc_results_internal(
    matches: &[Match],
    week: &i64,
    poolpicks: &[WeekPicks],
    picks: &HashMap<String, String>,
    poolerid: i64) -> u32 {

    let total = matches.iter().fold(0, |acc, m| {
        if let Some(pick) = picks.get(&m.id_event) {
            let pick = pick.as_str();
            let unique = poolpicks.iter()
                .filter(|&pp| pp.poolerid != poolerid)
                .map(|pp| { 
                    match &pp.picks {
                        Some(p) => p.get(&m.id_event).unwrap().as_str(),
                        None => "",
                    }
                })
                .all(|pp| pp != pick);

            let outcome = if let (Some(a), Some(h)) = (m.away_score, m.home_score) {
                match a.cmp(&h) {
                    Ordering::Less => if pick == m.away_team { MatchOutcome::Loss } else { MatchOutcome::Win },
                    Ordering::Greater => if pick == m.away_team { MatchOutcome::Win } else { MatchOutcome::Loss },
                    Ordering::Equal => MatchOutcome::Tied,
                }
            }
            else {
                MatchOutcome::NotPlayed
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
                160 => if unique { 6 } else { 4 },

                20 => if unique { 9 } else { 6 },
                125 => if unique { 9 } else { 6 },

                21 => if unique { 12 } else { 8 },
                150 => if unique { 12 } else { 8 },

                22 => if unique { 15 } else { 10 },
                200 => if unique { 15 } else { 10 },
                _ => 2,
            }
        },
        MatchOutcome::Loss | MatchOutcome::NotPlayed => 0,
        MatchOutcome::Tied => 1,
    }
}
