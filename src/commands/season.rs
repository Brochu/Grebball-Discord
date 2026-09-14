use std::env;
use std::collections::HashMap;
use std::fmt::Write;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{Match, calc_playoff_picture, calc_results, get_playoff_picture, get_week};

pub const ACCESS: i64 = 0;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("saison")
        .description("Montre les résultats de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
}

struct SeasonResult {
    //poolerid: i64,
    name: String,
    scores: Vec<u32>,
    cap_score: u32,
    total: u32,
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

    let (weeks, _week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    let capsule = match db.fetch_capsule(&season, &poolid).await {
        Ok(cap) => cap,
        Err(_) => HashMap::<_, _>::new(),
    };
    let picture = get_playoff_picture(season).await;
    let cap_results = if picture.reg_season_over {
        calc_playoff_picture(&picture, &capsule)
    } else {
        Vec::new()
    };
    let mut season_data = Vec::<SeasonResult>::new();

    for (week_num, feat, picks) in weeks.iter() {
        let matches: Vec<Match> = get_week(&season, week_num).await;
        for pick in picks {
            let score = if pick.cached.is_some() {
                pick.cached.unwrap() + pick.featcached.unwrap()
            } else {
                let results = calc_results(&pick.week, &matches, &picks, feat).await;
                let result = results.iter()
                    .find(|res| res.poolerid == pick.poolerid)
                    .unwrap();

                if result.cache {
                    db.cache_results(&result.pickid.unwrap(), &result.score, &result.featscore).await.unwrap();
                }
                result.score + result.featscore
            };

            if let Some(data) = season_data.iter_mut().find(|d| d.name.eq(&pick.name)) {
                data.scores.push(score);
                data.total += score;
            }
            else {
                let pooler_cap_score = match cap_results.iter().find(|cr| cr.poolerid == pick.poolerid) {
                    Some(res) => res.score,
                    None => 0,
                };
                season_data.push(SeasonResult{ name: pick.name.clone(), scores: vec![score], cap_score: pooler_cap_score, total: score });
            }
        }
    }

    season_data.sort_unstable_by(|l, r| {
        let r_full = r.total + r.cap_score;
        let l_full = l.total + l.cap_score;
        r_full.cmp(&l_full)
    });

    let mut message = String::new();
    write!(message, "{:<19}", "`Semaines").unwrap();
    for i in 1..=_week_count {
        if i <= 18 {
            write!(message, "|{:02}", i).unwrap();
        }
        else {
            match i {
                19 => write!(message, "|{:02}", "WC").unwrap(),
                20 => write!(message, "|{:02}", "DV").unwrap(),
                21 => write!(message, "|{:02}", "CF").unwrap(),
                22 => write!(message, "|{:02}", "SB").unwrap(),
                _ => unreachable!(),
            }
        }
    }
    if picture.reg_season_over {
        write!(message, "|+C").unwrap();
    }
    write!(message, "`").unwrap();

    // Only surface the capsule column once it actually counts (season over).
    for entry in season_data.iter() {
        write!(message, "\n`{:<12}[{:03}] ", entry.name, entry.total + entry.cap_score).unwrap();

        for s in entry.scores.iter() {
            write!(message, "|{:02}", s).unwrap();
        }

        if picture.reg_season_over {
            write!(message, "|{:02}`", entry.cap_score).unwrap();
        } else {
            write!(message, "`").unwrap();
        }
    }

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(format!("**Saison {}**\n{}\n", season, message))
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
