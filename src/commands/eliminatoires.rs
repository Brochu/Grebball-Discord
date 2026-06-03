use std::env;

use library::football;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::model::webhook::Webhook;
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
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content(format!("### Capsule {} — Correction\n", season).as_str())
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

    match env::var("RESULTS_WEBHOOK") {
        Ok(url) => {
            let hook = Webhook::from_url(&ctx.http, url.as_str()).await.unwrap();

            if results.is_empty() {
                hook.execute(&ctx.http, false, |h| { h.content("Aucune capsule trouvée.") }).await.unwrap();
                return;
            }

            for (i, r) in results.iter().enumerate() {
                let pad = " ".repeat(12usize.saturating_sub(r.name.len()));
                hook.execute(&ctx.http, false, |h| {
                    h.content(format!("`#{:<2} {}{} {:>3}pts` {}\n", i+1, r.name, pad, r.score, r.icons).as_str())
                }).await.unwrap();
            }
            // TODO: persist scores via cache_capsule once a season-over gate exists
        },
        Err(_) => { panic!("Could not find ENV `RESULTS_WEBHOOK`") },
    }
}
