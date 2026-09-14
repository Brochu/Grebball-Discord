use std::env;

use library::football;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;

pub const ACCESS: i64 = 1;

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
                .content("Calcul ...")
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

    let picture = football::get_playoff_picture(season).await;
    let results = football::calc_playoff_picture(&picture, &capsules);

    if results.is_empty() {
        if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
            res.content("Aucune capsule trouvée.")
        }).await {
            println!("![eliminatoires] Cannot edit interaction response : {:?}", reason);
        }
        return;
    }

    // Padded to the same width as the pooler rows, so the emoji grids line up.
    let header = format!("### Capsule {} — Correction\n`{:<23}` {}",
        season, "Résultats", football::format_playoff_picture(&picture));

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(header)
    }).await {
        println!("![eliminatoires] Cannot edit interaction response : {:?}", reason);
    }

    for (i, r) in results.iter().enumerate() {
        if let Err(message) = command.channel_id.send_message(&ctx.http, |res| {
            res.content(format!("`#{:<2} {:<12} {:>3}pts` {}\n", i+1, r.name, r.score, r.icons).as_str())
        }).await {
            println!("![eliminatoires] Cannot respond to interaction : {:?}", message);
        }
    }
}
