use std::env;

use serde_json::Value;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use library::football::{ get_week, get_team_emoji };

const VS_EMOJI: &str = "<:VS:1144451849571794997>";

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("semaine")
        .description("Montre tous les matches d'une semaine")
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
                .add_string_choice("WildCards", 160)
                .add_string_choice("Divisional", 125)
                .add_string_choice("Championship", 150)
                .add_string_choice("Super Bowl", 200)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    if let Value::String(str) = command.data.options.get(0)
        .expect("![Week] Could not fetch week arg")
        .value.as_ref()
        .expect("![Week] Could not get value of the week arg")
    {
        if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
            res
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m|
                    m.content("Calcul ...")
                )
        })
        .await {
            println!("![week] Cannot respond to slash command : {:?}", reason);
        }

        let season = env::var("CONF_SEASON")
            .expect("![Week] Could not find 'CONF_SEASON' env var")
            .parse::<u16>()
            .expect("![Week] Could not parse 'CONF_SEASON' to int");
        let week = str.parse::<i64>()
            .expect("![Week] Could not parse week arg to u64");
        let matches = get_week(&season, &week).await
            .expect("![Week] Could not fetch match data");

        let output = matches.fold(String::new(), |mut out, m| {
            let aemoji = get_team_emoji(m.away_team.as_str());
            let hemoji = get_team_emoji(m.home_team.as_str());

            let (ascore, hscore, aline, hline) = if let (Some(a), Some(h)) = (m.away_score, m.home_score) {
                (a.to_string(), h.to_string(), a > h, h > a)
            }
            else {
                ("--".to_string(), "--".to_string(), false, false)
            };

            out.push_str(format!("<:{}:{}> {} {} {} <:{}:{}>\n",
                m.away_team, aemoji,
                if aline { format!("__`{:02}`__", ascore) } else { format!("`{:02}`", ascore) },
                VS_EMOJI,
                if hline { format!("__`{:02}`__", hscore) } else { format!("`{:02}`", hscore) },
                m.home_team, hemoji
            ).as_str());
            out
        });

        if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
            res.content(output)
        })
        .await {
            println!("![week] Cannot respond to slash command : {:?}", reason);
        }
    }
}
