use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
//use library::football::{calc_results, get_week, Match};

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
    total: u32,
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");

    let mut season_data = db.fetch_season(&poolid, &season).await.unwrap()
        .iter()
        .fold(Vec::<SeasonResult>::new(), |mut season, (_, data)| {
            let name = data[0].name.to_owned();

            let scores: Vec<u32> = data.iter().enumerate().map(|(_i, wp)| {
                if let Some(score) = wp.cached {
                    score
                }
                else {
                    0
                }
            })
            .collect();

            let total = scores.iter().sum();

            season.push(SeasonResult {
                //poolerid: *poolerid,
                name,
                scores,
                total
            });

            season
        });

    //for week in vec!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 160, 125, 150, 200) {
    //    //TODO: Can we make this better? Like bulk operations?
    //    let matches: Vec<Match> = get_week(&season, &week).await
    //        .unwrap().collect();
    //    let picks = db.fetch_picks(&poolid, &season, &week).await
    //        .expect("![season] Could not fetch picks to complete season command");

    //    let results = calc_results(&week, &matches, &picks).await;
    //    if results.iter().all(|res| res.score == 0) {
    //        break;
    //    }

    //    for res in results {
    //        if res.cache {
    //            db.cache_results(&res.pickid.unwrap(), &res.score).await.unwrap();
    //        }

    //        let poolerid = res.poolerid;

    //        if let Some(entry) = season_data.iter_mut().find(|d| d.poolerid == poolerid) {
    //            entry.scores.push(res.score);

    //            let newscore: u64 = res.score.into();
    //            entry.total += newscore;
    //        }
    //        else {
    //            season_data.push(SeasonResult {
    //                poolerid,
    //                name: format!("{}", res.name),
    //                scores: vec![res.score],
    //                total: res.score.into(),
    //            });
    //        }
    //    }
    //}

    season_data.sort_unstable_by(|l, r| { r.total.cmp(&l.total) });
    let message = season_data.iter()
        .fold(String::new(), |m, entry| {
            let width = 10 - entry.name.len();
            let grid = entry.scores.iter().fold(String::new(), |g, s| { format!("{}| `{:02}` ", g, s) });

            format!("{}\n`{}{}[{:03}]`: {}", m, entry.name, " ".repeat(width), entry.total, grid)
        });

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content(format!("Saison {}{}", season, message))
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
