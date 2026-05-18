use std::{env, cmp::Ordering, collections::HashMap};
use core::fmt::{Display, Debug};

use chrono::{ DateTime, TimeDelta, Utc };
use serde::{Deserialize, Serialize};
use serenity::model::id::EmojiId;

use crate::database::{WeekFeature, WeekPicks, CapsulePicks};

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

//TODO: This might be better as a match on enum FeatPick::Over vs. FeatPick::Under
pub fn get_overpick_emoji() -> EmojiId {
    EmojiId(1415070599415468096)
}
pub fn get_underpick_emoji() -> EmojiId {
    EmojiId(1415070657682870312)
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

//TODO: Return vector<Match> here so we can collect and sort matches here
// ESPN's matches get re-ordered based on some logic I don't understand yet
// I think the outputs on Discord look better when the matches are always in the same order
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
    pub featscore: u32,
    pub icons: String,
    pub overunder: String,
    pub cache: bool,
}

impl Display for PickResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {} - {} + {} (should cache? {})",
            self.pickid, self.name, self.score, self.featscore, self.cache)
    }
}

pub async fn calc_results(week: &i64, matches: &[Match], picks: &[WeekPicks], feat: &Option<WeekFeature>) -> Vec<PickResults> {
    let now = Utc::now().checked_sub_signed(TimeDelta::hours(8)).unwrap();
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
            let mut featscore: u32 = 0;

            let icons = if let Some(picks) = &p.picks {
                matches.iter().fold(String::new(), |mut acc, m| {
                    if let (Some(f), Some(fp), Some(asc), Some(hsc)) = (feat, p.featpick, m.away_score, m.home_score) {
                        //TODO: Can we make this more generic if needed?
                        if m.id_event == f.matchid && (asc + hsc) > 0 && (
                            fp == 1 && (asc + hsc) > f.target as u64 ||
                            fp == 0 && (asc + hsc) <= f.target as u64
                        ) {
                            featscore = 3;
                        }
                    }
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

            let overunder = if let Some(featpick) = &p.featpick {
                if *featpick == 0 {
                    format!(":{}:", "chart_with_downwards_trend")
                } else {
                    format!(":{}:", "chart_with_upwards_trend")
                }
            } else {
                String::new()
            };

            if let Some(cached) = p.cached {
                PickResults {
                    pickid: p.pickid, poolerid: p.poolerid, name,
                    score: cached, featscore: p.featcached.unwrap_or_else(|| featscore), icons, overunder,
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
                PickResults {
                    pickid: p.pickid, poolerid: p.poolerid,
                    name, score, featscore, icons, overunder, cache
                }
            }
        })
        .collect();

    output.sort_unstable_by(|l, r| {
        let rs = r.score + r.featscore;
        let ls = l.score + l.featscore;
        rs.cmp(&ls)
    });
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
				if a == 0 && h == 0 {
					MatchOutcome::NotPlayed
				} else {
					match a.cmp(&h) {
						Ordering::Less => if pick == m.away_team { MatchOutcome::Loss } else { MatchOutcome::Win },
						Ordering::Greater => if pick == m.away_team { MatchOutcome::Win } else { MatchOutcome::Loss },
						Ordering::Equal => MatchOutcome::Tied,
					}
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
                1..=18 => if unique { 4 } else { 2 },
                19 => if unique { 6 } else { 4 },
                160 => if unique { 6 } else { 4 },

                20 => if unique { 8 } else { 6 },
                125 => if unique { 8 } else { 6 },

                21 => if unique { 10 } else { 8 },
                150 => if unique { 10 } else { 8 },

                22 => if unique { 12 } else { 10 },
                200 => if unique { 12 } else { 10 },
                _ => if unique { 4 } else { 2 },
            }
        },
        MatchOutcome::Loss | MatchOutcome::NotPlayed => 0,
        MatchOutcome::Tied => 1,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct ESPNTeam {
    abbreviation: String,
    displayName: String,
}

#[derive(Deserialize, Debug)]
struct ESPNStandings {
    children: Vec<ESPNConference>,
}

#[derive(Deserialize, Debug)]
struct ESPNConference {
    abbreviation: String,
    children: Vec<ESPNDivision>,
}

#[derive(Deserialize, Debug)]
struct ESPNDivision {
    abbreviation: String,
    standings: ESPNStandingsBody,
}

#[derive(Deserialize, Debug)]
struct ESPNStandingsBody {
    entries: Option<Vec<ESPNStandingsEntry>>,
}

#[derive(Deserialize, Debug)]
struct ESPNStandingsEntry {
    team: ESPNTeam,
    stats: Vec<ESPNStat>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ESPNStat {
    name: String,
    displayValue: String,
}

#[derive(Debug, Default)]
pub struct PlayoffPicture {
    pub afc_n_win: String,
    pub afc_s_win: String,
    pub afc_e_win: String,
    pub afc_w_win: String,
    pub nfc_n_win: String,
    pub nfc_s_win: String,
    pub nfc_e_win: String,
    pub nfc_w_win: String,
    pub afc_wildcards: Vec<String>,
    pub nfc_wildcards: Vec<String>,
}

pub async fn get_playoff_picture(season: u16) -> PlayoffPicture {
    let standings_url = env::var("STANDINGS_URL")
        .expect("![Football] Could not find 'STANDINGS_URL' env var");

    let url = format!("{}?season={}&type=0&level=3", standings_url, season);
    let res = reqwest::get(url).await
        .expect("![Football] Could not get standings reply")
        .text().await
        .expect("![Football] Could not retrieve standings text");
    let standings: ESPNStandings = serde_json::from_str(&res)
        .expect("![Football] Could not parse standings response");

    let mut picture = PlayoffPicture::default();

    for conference in &standings.children {
        let conf = conference.abbreviation.as_str();
        let mut wildcards: Vec<(u32, String)> = Vec::new();

        for division in &conference.children {
            let div_abbr = division.abbreviation.as_str();

            let entries = match &division.standings.entries {
                Some(entries) => entries,
                None => continue,
            };

            for entry in entries {
                let seed = entry.stats.iter()
                    .find(|s| s.name == "playoffSeed")
                    .and_then(|s| s.displayValue.parse::<u32>().ok());

                if let Some(seed) = seed {
                    let team = entry.team.abbreviation.clone();

                    if (1..=4).contains(&seed) {
                        match (conf, div_abbr) {
                            ("AFC", "NORTH") => picture.afc_n_win = team,
                            ("AFC", "SOUTH") => picture.afc_s_win = team,
                            ("AFC", "EAST")  => picture.afc_e_win = team,
                            ("AFC", "WEST")  => picture.afc_w_win = team,
                            ("NFC", "NORTH") => picture.nfc_n_win = team,
                            ("NFC", "SOUTH") => picture.nfc_s_win = team,
                            ("NFC", "EAST")  => picture.nfc_e_win = team,
                            ("NFC", "WEST")  => picture.nfc_w_win = team,
                            _ => {}
                        }
                    } else if (5..=7).contains(&seed) {
                        wildcards.push((seed, team));
                    }
                }
            }
        }

        wildcards.sort_by_key(|(seed, _)| *seed);
        let wc_teams: Vec<String> = wildcards.into_iter().map(|(_, t)| t).collect();

        match conf {
            "AFC" => picture.afc_wildcards = wc_teams,
            "NFC" => picture.nfc_wildcards = wc_teams,
            _ => {}
        }
    }

    return picture;
}

#[derive(Debug, Default)]
pub struct CapsuleResults {
    pub poolerid: i64,
    pub name: String,
    pub score: u32,
    pub icons: String,
    pub cache: bool,
}

#[derive(Debug)]
enum CapsuleState {
    DivisionWinner,
    Wildcard,
}

pub async fn calc_playoff_picture(picture: &PlayoffPicture, picks: &[CapsulePicks]) -> Vec<CapsuleResults> {
    // (accessor for a pooler's pick in this division, actual division winner).
    // Non-capturing closures coerce to fn pointers so they can share one array type.
    type DivField = fn(&CapsulePicks) -> Option<&String>;
    let divisions: [(DivField, &str); 8] = [
        (|c| c.winafcn.as_ref(), picture.afc_n_win.as_str()),
        (|c| c.winafcs.as_ref(), picture.afc_s_win.as_str()),
        (|c| c.winafce.as_ref(), picture.afc_e_win.as_str()),
        (|c| c.winafcw.as_ref(), picture.afc_w_win.as_str()),
        (|c| c.winnfcn.as_ref(), picture.nfc_n_win.as_str()),
        (|c| c.winnfcs.as_ref(), picture.nfc_s_win.as_str()),
        (|c| c.winnfce.as_ref(), picture.nfc_e_win.as_str()),
        (|c| c.winnfcw.as_ref(), picture.nfc_w_win.as_str()),
    ];

    let mut results = Vec::<CapsuleResults>::new();
    for pick in picks {
        let mut score = 0;
        let mut icons = String::new();

        for (field, actual) in &divisions {
            // Skip divisions with no determined winner yet (avoids "" == "" false positives).
            if actual.is_empty() { continue; }

            let predicted = field(pick).map(String::as_str).unwrap_or_default();
            icons.push_str(format!("<:{}:{}>", predicted, get_team_emoji(predicted)).as_str());

            if predicted != *actual { continue; }

            // Unique = no other pooler picked the same team for this division.
            // Mirrors the per-match uniqueness check in calc_results_internal.
            let unique = picks.iter()
                .filter(|p| p.poolerid != pick.poolerid)
                .all(|p| field(p).map(String::as_str).unwrap_or_default() != predicted);

            score += get_capsule_score(CapsuleState::DivisionWinner, unique);
        }
        icons.push_str("||");

        if let Some(afc_wildcard) = &pick.afcwildcards {
            for team in afc_wildcard.split(",") {
                icons.push_str(format!("<:{}:{}>", team, get_team_emoji(team)).as_str());
                if picture.afc_wildcards.iter().any(|t| t == team) { score += get_capsule_score(CapsuleState::Wildcard, false); }
            }
        }

        if let Some(nfc_wildcard) = &pick.nfcwildcards {
            for team in nfc_wildcard.split(",") {
                icons.push_str(format!("<:{}:{}>", team, get_team_emoji(team)).as_str());
                if picture.nfc_wildcards.iter().any(|t| t == team) { score += get_capsule_score(CapsuleState::Wildcard, false); }
            }
        }

        results.push(CapsuleResults {
            poolerid: pick.poolerid,
            name: "".to_owned(),
            score,
            icons,
            cache: false,
        });
    }

    return results;
}

fn get_capsule_score(state: CapsuleState, unique: bool) -> u32 {
    match (state, unique) {
        (CapsuleState::DivisionWinner, true)  => 4,
        (CapsuleState::DivisionWinner, false) => 2,
        (CapsuleState::Wildcard      , true)  => 2,
        (CapsuleState::Wildcard      , false) => 1,
    }
}
