use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("eliminatoires")
        .description("Corriger les capsules de tous les poolers pour la saison en cours")
        .kind(CommandType::ChatInput)
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![eliminatoires] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![eliminatoires] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[eliminatoires] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[eliminatoires] Could not parse 'CONF_SEASON' to u16");

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Correction ...")
            )
    }).await {
        println!("![eliminatoires] Cannot respond to slash command : {:?}", reason);
    }

    let capsules = match db.fetch_capsule(&season, &poolid).await {
        Ok(c) => c,
        Err(e) => {
            println!("![eliminatoires] Could not fetch capsules for poolid: {}, season: {}\nerror: {}", poolid, season, e);
            if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
                res.content("Une erreur s'est produite lors de la récupération des capsules.")
            }).await {
                println!("![eliminatoires] Cannot edit interaction response : {:?}", reason);
            }
            return;
        }
    };

    // TODO: score each capsule and persist results

    let content = format!("Correction des capsules pour la saison {} ({} poolers trouvés).", season, capsules.len());

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(content)
    }).await {
        println!("![eliminatoires] Cannot edit interaction response : {:?}", reason);
    }
}
