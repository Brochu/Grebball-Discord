use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{Match, get_week, calc_results};

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

    let (poolers, week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    let mut season_data = Vec::<SeasonResult>::new();

    for (poolerid, weeks) in poolers.iter() {
        let name = weeks[0].name.to_owned();
        let mut scores = Vec::<u32>::new();
        let mut total = 0;

        for i in 0..week_count {
            if let Some(w) = weeks.get(i) {
                if let Some(score) = w.cached {
                    scores.push(score);
                    total += score;
                }
                else {
                    let week: i64 = (i + 1).try_into().unwrap();
                    let matches: Vec<Match> = get_week(&season, &week).await.unwrap().collect();
                    let picks = db.fetch_pick(&poolid, &season, &week, &poolerid).await.unwrap();

                    let result = &calc_results(&week, &matches, &[picks]).await[0];
                    if result.cache {
                        db.cache_results(&result.pickid.unwrap(), &result.score).await.unwrap();
                    }

                    scores.push(result.score);
                    total += result.score;
                }
            }
            else {
                scores.push(0);
            }
        }

        season_data.push(SeasonResult {
            name,
            scores,
            total
        });
    }
    season_data.iter().for_each(|data| {
        println!("[{}]: {:?} = {}", data.name, data.scores, data.total);
    });

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
