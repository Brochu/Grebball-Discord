use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("stats")
        .description("Montre les statistiques de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
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

    let (poolers, _week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    let _stats = poolers.iter()
        .map(|(_id, _picks)| {
            String::new()
        })
        .inspect(|s| println!("{}", s))
        .collect::<Vec<String>>();

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(format!("Statistiques {}-{}\n{}", season, poolid, "STATS HERE"))
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
