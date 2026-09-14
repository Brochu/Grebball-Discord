use std::env;
use std::cmp::Reverse;
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
    poolerid: i64,
    name: String,
    scores: Vec<u32>,
    cap_score: u32,
    total: u32,
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![season] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![season] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[season] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[season] Could not parse 'CONF_SEASON' to u16");

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Calcul ...")
            )
    })
    .await {
        println!("![season] Cannot respond to slash command : {:?}", reason);
    }

    let (weeks, week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    let picture = get_playoff_picture(season).await;
    let cap_results = if picture.reg_season_over {
        let capsule = db.fetch_capsule(&season, &poolid).await.unwrap_or_default();
        calc_playoff_picture(&picture, &capsule)
    } else {
        Vec::new()
    };
    let mut season_data = Vec::<SeasonResult>::new();

    for (week_num, feat, picks) in weeks.iter() {
        let mut results = Vec::new();
        for pick in picks {
            let score = if let Some(cached) = pick.cached {
                cached + pick.featcached.unwrap()
            } else {
                if results.is_empty() {
                    let matches: Vec<Match> = get_week(&season, week_num).await;
                    results = calc_results(week_num, &matches, &picks, feat).await;
                }
                let result = results.iter()
                    .find(|res| res.poolerid == pick.poolerid)
                    .unwrap();

                if result.cache {
                    db.cache_results(&result.pickid.unwrap(), &result.score, &result.featscore).await.unwrap();
                }
                result.score + result.featscore
            };

            if let Some(data) = season_data.iter_mut().find(|d| d.poolerid == pick.poolerid) {
                data.scores.push(score);
                data.total += score;
            }
            else {
                let pooler_cap_score = match cap_results.iter().find(|cr| cr.poolerid == pick.poolerid) {
                    Some(res) => res.score,
                    None => 0,
                };
                season_data.push(SeasonResult{ poolerid: pick.poolerid, name: pick.name.clone(), scores: vec![score], cap_score: pooler_cap_score, total: score + pooler_cap_score });
            }
        }
    }

    season_data.sort_unstable_by_key(|d| Reverse(d.total));

    let mut message = String::new();
    for (first, last) in [(1, 9), (10, 18), (19, 22)] {
        // Only surface the capsule column once it actually counts (season over).
        let with_cap = first == 19 && picture.reg_season_over;
        if first != 1 && first > week_count && !with_cap {
            continue;
        }
        let last = last.min(week_count);

        message.clear();
        if first == 1 {
            write!(message, "**Saison {}**\n", season).unwrap();
        }
        write!(message, "{:<19}", "`Semaines").unwrap();
        for i in first..=last {
            match i {
                1..=18 => write!(message, "|{:02}", i).unwrap(),
                19 => message.push_str("|WC"),
                20 => message.push_str("|DV"),
                21 => message.push_str("|CF"),
                22 => message.push_str("|SB"),
                _ => unreachable!(),
            }
        }
        if with_cap {
            write!(message, "|+C").unwrap();
        }
        write!(message, "`").unwrap();

        for entry in season_data.iter() {
            write!(message, "\n`{:<12}[{:03}] ", entry.name, entry.total).unwrap();

            for s in entry.scores.iter().skip(first - 1).take(last + 1 - first) {
                write!(message, "|{:02}", s).unwrap();
            }

            if with_cap {
                write!(message, "|{:02}", entry.cap_score).unwrap();
            }
            write!(message, "`").unwrap();
        }

        if first == 1 {
            message.push('\n');
            if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
                res.content(&message)
            })
            .await {
                println!("![season] Cannot respond to slash command : {:?}", reason);
            }
        }
        else if let Err(reason) = command.channel_id.send_message(&ctx.http, |res| {
            res.content(&message)
        })
        .await {
            println!("![season] Cannot respond to slash command : {:?}", reason);
        }
    }
}
