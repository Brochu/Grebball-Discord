use std::collections::HashMap;
use std::{env, vec};

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{calc_results, get_week, Match};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("saison")
        .description("Montre les résultats de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");

    let mut season_data: HashMap<String, (Vec<u32>, u64)> = HashMap::new();

    for week in vec!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 160, 125, 150, 200) {
        //TODO: Can we make this better? Like bulk operations?
        let matches: Vec<Match> = get_week(&season, &week).await
            .unwrap().collect();
        let picks = db.fetch_picks(&poolid, &season, &week).await
            .expect("![season] Could not fetch picks to complete season command");

        let results = calc_results(&week, &matches, &picks).await;
        if results.iter().all(|res| res.score == 0) {
            break;
        }

        //TODO: Can we access the poolerid in any way here?
        results.iter().for_each(|res| {
            if let Some(entry) = season_data.get_mut(&res.name) {
                entry.0.extend(vec![res.score]);

                let newscore: u64 = res.score.into();
                entry.1 += newscore;
            }
            else {
                season_data.insert(format!("{}", res.name), (
                    vec![res.score],
                    res.score.into()
                ));
            }
        });
    }
    season_data.iter().for_each(|(k, v)| {
        println!("[{}] -> {:?}", k, v);
    });

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Here are the results for the current season ...")
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
