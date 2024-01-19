use std::env;
use serde_json::{Value, Map};

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::{DB, PicksStatus};
use library::football::{get_week, get_team_emoji};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("choix")
        .description("Faire ses choix pour une semaine de la saison courante")
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
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let season = env::var("CONF_SEASON")
        .expect("[picks] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[picks] Could not parse 'CONF_SEASON' to u16");
    let week = command.data.options.first()
        .expect("[picks] No week arg given with the command").value.as_ref()
        .unwrap().as_str().unwrap().parse::<i64>()
        .expect("[picks] Could not parse week arg to u64");
    let week_name = command.data.options.first().unwrap().name.as_str();

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    if let Ok(status) = db.prime_picks(&discordid, &season, &week).await {
        match status {
            PicksStatus::Primed(row_id) => {
                let picks_url = env::var("PICKS_URL")
                    .expect("![Picks] Could not find 'PICKS_URL' env var");
                let url = format!("{}/{}/{}", picks_url, discordid, row_id);

                if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                    res
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m
                            .ephemeral(true)
                            .content(format!("Prêt pour les choix de la {} à faire ici: {}", week_name, url))
                        )
                })
                .await {
                    println!("![picks] Cannot respond to slash command : {:?}", reason);
                }
            },
            PicksStatus::Filled(pickstring) => {
                let picks: Map<String, Value> = serde_json::from_str(&pickstring)
                    .expect("![picks] Could not parse picks properly");

                let icons = get_week(&season, &week).await
                    .expect("![picks] Could not fetch week data")
                    .fold(String::new(), |mut acc, m| {
                        let team = picks.get(&m.id_event).unwrap()
                            .as_str().unwrap();
                        let emoji = get_team_emoji(team);

                        acc.push_str(format!("<:{}:{}> ", team, emoji).as_str());
                        acc
                    });

                if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                    res
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m
                            .ephemeral(true)
                            .content(format!("Choix pour la semaine {}, {}\n{}", week, season, icons))
                        )
                })
                .await {
                    println!("![picks] Cannot respond to slash command : {:?}", reason);
                }
            },
        }
    } else {
        println!("[picks] Error while priming picks for week {}", week);

        if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
            res
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m
                    .ephemeral(true)
                    .content("Une erreur s'est produite avec la commande `/choix` .")
                )
        })
        .await {
            println!("![picks] Cannot respond to slash command : {:?}", reason);
        }
    }
}
