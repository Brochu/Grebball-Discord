use std::env;
use std::fmt::Display;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{ Match, get_week };

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("stats")
        .description("Montre les statistiques de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
}

struct PoolStats {
    uni_count: u32,
    uni_hits: u32,
    unique_count: u32,
    unique_hits: u32,
}
impl Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[POOL] unilateral rate: {} / {}; unique rate: {} / {}",
            self.uni_hits, self.uni_count, self.unique_hits, self.unique_count)
    }
}

struct PoolerStats {
    name: String,
    pick_count: u32,
    hit_count: u32,
    unique_count: u32,
    unique_hits: u32,
}
impl Display for PoolerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] hit rate: {} / {}; unique hit rate: {} / {}",
            self.name, self.hit_count, self.pick_count, self.unique_hits, self.unique_count)
    }
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Calcul ...")
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }

    let mut pool = PoolStats{ uni_count: 0, uni_hits: 0, unique_count: 0, unique_hits: 0 };
    let mut stats = Vec::<PoolerStats>::new();

    let (weeks, _) = db.fetch_season(&poolid, &season).await.unwrap();
    for (w, _feat_id, poolers) in &weeks[..] {
        for m in get_week(&season, &w).await.unwrap() {
            //TODO: Look into skipping matches that are not played yet
            let picks: Vec<(_, _)> = poolers.iter()
                .map(|p| {
                    let pick = match &p.picks {
                        Some(pick) => pick.get(&m.id_event).unwrap().as_str(),
                        None => "",
                    };
                    (p.name.as_str(), pick)
                })
                .inspect(|&(name, pick)| {
                    let win = pick == &m.away_team && m.away_score > m.home_score ||
                              pick == &m.home_team && m.home_score > m.away_score;

                    if let Some(stat) = stats.iter_mut().find(|s| &s.name == name) {
                        stat.pick_count += 1;
                        if win {
                            stat.hit_count += 1;
                        }
                    } else {
                        stats.push(PoolerStats {
                            name: name.to_owned(),
                            pick_count: 1,
                            hit_count: if win { 1 } else { 0 },
                            unique_count: 0,
                            unique_hits: 0
                        });
                    }
                })
                .collect();
            check_unanimous(&m, &picks, &mut pool.uni_hits, &mut pool.uni_count);
            check_unique(&m, &picks, &mut pool.unique_hits, &mut pool.unique_count, &mut stats);
        }
    }

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        let uni = format!(" - Choix unanimes: {}/{} ({:.2}%)",
            pool.uni_hits, pool.uni_count, (pool.uni_hits as f32 / pool.uni_count as f32) * 100.0);
        let unique = format!("- Choix uniques: {}/{} ({:.2}%)",
            pool.unique_hits, pool.unique_count, (pool.unique_hits as f32 / pool.unique_count as f32) * 100.0);
        let list = stats.iter()
            .fold(String::new(), |message, stat| {
                let unique_percent = if stat.unique_count > 0 {
                    stat.unique_hits as f32 / stat.unique_count as f32 * 100.0
                } else {
                    0.0
                };

                let pooler = format!("### {}\n - Choix: {}/{} ({:.2}%)\n- Uniques: {}/{} ({:.2}%)",
                    stat.name,
                    stat.hit_count, stat.pick_count, (stat.hit_count as f32 / stat.pick_count as f32) * 100.0,
                    stat.unique_hits, stat.unique_count, unique_percent);
                format!("{}\n{}", message, pooler)
            });

        res.content(format!("## Statistiques de la saison {}\n{}\n{}\n{}", season, uni, unique, list))
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}

fn check_unanimous(m: &Match, picks: &Vec<(&str, &str)>, uni_hit: &mut u32, uni_count: &mut u32) {
    let all_away = picks.iter().all(|&(_, p)| p == m.away_team);
    let all_home = picks.iter().all(|&(_, p)| p == m.home_team);

    if all_away || all_home {
        *uni_count += 1;

        if all_away && m.away_score > m.home_score || all_home && m.home_score > m.away_score {
            *uni_hit += 1;
        }
    }
}

fn check_unique(
    m: &Match,
    picks: &Vec<(&str, &str)>,
    uni_hit: &mut u32,
    uni_count: &mut u32,
    stats: &mut Vec<PoolerStats>)
{
    let away_count = picks.iter().filter(|(_, pick)| pick == &m.away_team).count();
    let home_count = picks.iter().filter(|(_, pick)| pick == &m.home_team).count();

    if away_count == 1 || home_count == 1 {
        *uni_count += 1;

        let name = if away_count < home_count {
            picks.iter().find(|(_, pick)| pick == &m.away_team).unwrap().0
        } else {
            picks.iter().find(|(_, pick)| pick == &m.home_team).unwrap().0
        };

        let stat = stats.iter_mut().find(|s| &s.name == name).unwrap();
        stat.unique_count += 1;

        if away_count == 1 && m.away_score > m.home_score || home_count == 1 && m.home_score > m.away_score {
            *uni_hit += 1;
            stat.unique_hits += 1;
        }
    }
}
