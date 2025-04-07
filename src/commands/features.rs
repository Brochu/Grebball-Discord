use std::env;

use serde_json::Value;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use library::database::DB;
use library::football::get_week;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("features")
        .description("Choisir le match featured pour une semaine")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("semaine")
                .kind(CommandOptionType::String)
                .description("La semaine choisie")
                .required(true)
                .add_string_choice("semaine 1", 1)
                .add_string_choice("semaine 2", 2)
                .add_string_choice("semaine 3", 3)
                .add_string_choice("semaine 4", 4)
                .add_string_choice("semaine 5", 5)
                .add_string_choice("semaine 6", 6)
                .add_string_choice("semaine 7", 7)
                .add_string_choice("semaine 8", 8)
                .add_string_choice("semaine 9", 9)
                .add_string_choice("semaine 10", 10)
                .add_string_choice("semaine 11", 11)
                .add_string_choice("semaine 12", 12)
                .add_string_choice("semaine 13", 13)
                .add_string_choice("semaine 14", 14)
                .add_string_choice("semaine 15", 15)
                .add_string_choice("semaine 16", 16)
                .add_string_choice("semaine 17", 17)
                .add_string_choice("semaine 18", 18)
                .add_string_choice("WildCards", 19)
                .add_string_choice("Divisional", 20)
                .add_string_choice("Championship", 21)
                .add_string_choice("Super Bowl", 22)
        })
        .create_option(|opt| {
            opt
                .name("cible")
                .kind(CommandOptionType::Integer)
                .description("valeur cible pour le match featured")
                .required(true)
        })
        .create_option(|opt| {
            opt
                .name("match")
                .kind(CommandOptionType::Integer)
                .description("index du match à choisir comme featured")
                .required(true)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let week_opt = command.data.options.get(0);
    let target_opt = command.data.options.get(1);
    let match_opt = command.data.options.get(2);

    let season = env::var("CONF_SEASON")
        .expect("![Week] Could not find 'CONF_SEASON' env var")
        .parse::<u16>()
        .expect("![Week] Could not parse 'CONF_SEASON' to int");
    let week = str.parse::<i64>()
        .expect("![Week] Could not parse week arg to u64");
    let matches = get_week(&season, &week).await
        .expect("![Week] Could not fetch match data");
    let match_id = String::new();

    match db.set_feature(season, week, 0, "".to_owned()).await {
        Ok(out) => {
        },

        Err(out) => {
        },
    };

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                .content(format!("Choisi match {} comme featured pour saison: {} semaine {}",
                        match_id, season, week))
            )
    })
    .await {
        println!("![features] Cannot respond to slash command : {:?}", reason);
    }
}
