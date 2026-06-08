use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;
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

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let poolerid = match db.fetch_poolerid(&discordid).await {
        Ok(pid) => pid,
        Err(_) => {
            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .ephemeral(true)
                        .content("Tu n'es pas inscrit au pool.")
                    )
            })
            .await {
                println!("![picks] Cannot respond to slash command : {:?}", reason);
            }
            return;
        },
    };

    let message = match db.fetch_pick(&season, &week, &poolerid).await {
        Ok(p) => {
            let picks = p.picks.unwrap();
            let feature = db.fetch_feature(season, week).await.ok();

            let (icons, feat_str) = get_week(&season, &week).await
                .into_iter()
                .fold((String::new(), String::new()), |(mut icons, mut feat_str), m| {
                    let team = picks.get(&m.id_event).unwrap();
                    let emoji = get_team_emoji(team);

                    icons.push_str(format!("<:{}:{}> ", team, emoji).as_str());

                    if let Some(ref feat) = feature {
                        if feat.matchid == m.id_event {
                            let away_emoji = get_team_emoji(&m.away_team);
                            let home_emoji = get_team_emoji(&m.home_team);
                            let trend = match p.featpick {
                                Some(1) => ":chart_with_upwards_trend:",
                                _ => ":chart_with_downwards_trend:",
                            };
                            feat_str = format!(
                                "<:{}:{}> @ <:{}:{}> {}",
                                m.away_team, away_emoji, m.home_team, home_emoji, trend
                            );
                        }
                    }

                    (icons, feat_str)
                });

            if feat_str.is_empty() {
                format!("## Choix pour la semaine {}, {}\n{}", week, season, icons)
            } else {
                format!("## Choix pour la semaine {}, {}\n{}\n**Feature:** {}", week, season, icons, feat_str)
            }
        },
        Err(_) => match db.issue_pick_token(season, week, poolerid).await {
            Ok(token) => {
                let picks_url = env::var("PICKS_URL").expect("![Picks] Could not find 'PICKS_URL' env var");
                let week_name = command.data.options.first().unwrap().name.as_str();
                format!("Prêt pour les choix de la {} à faire ici: {}/{}", week_name, picks_url, token)
            },
            Err(_) => "Une erreur s'est produite avec la commande `/choix` .".to_string(),
        },
    };

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                .content(message)
            )
    })
    .await {
        println!("![picks] Cannot respond to slash command : {:?}", reason);
    }
}
