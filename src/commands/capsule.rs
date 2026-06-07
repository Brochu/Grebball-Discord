use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{get_team_emoji, get_afc_emoji, get_nfc_emoji};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("capsule")
        .description("Faire ses prédictions pour la capsule de l'année")
        .kind(CommandType::ChatInput)
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let season = env::var("CONF_SEASON")
        .expect("[capsule] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[capsule] Could not parse 'CONF_SEASON' to u16");

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

    match db.fetch_pooler_capsule(&discordid, season).await {
        Ok(None) => {
            let message = match db.issue_pick_token(season, 0, poolerid).await {
                Ok(token) => {
                    let picks_url = env::var("PICKS_URL")
                        .expect("![capsule] Could not find 'PICKS_URL' env var");
                    let url = format!("{}/capsule/{}", picks_url, token);

                    format!("Prêt pour les prédictions de la capsule {} à faire ici: {}", season, url)
                },
                Err(_) => "Une erreur s'est produite avec la commande `/capsule` .".to_string(),
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
                println!("![capsule] Cannot respond to slash command : {:?}", reason);
            }
        },
        Ok(Some(capsule)) => {
            let mut content = format!("**Capsule {}**\n\n", season);

            content.push_str(&format!("<:AFC:{}> **Gagnants -** :", get_afc_emoji()));
            for team in &capsule.afc_wins {
                content.push_str(&format!(" <:{}:{}>", team, get_team_emoji(team)));
            }
            content.push_str(&format!("  /  **Wildcards -** :"));
            for team in &capsule.afc_wildcards {
                content.push_str(&format!(" <:{}:{}>", team, get_team_emoji(team)));
            }

            content.push_str(&format!("\n<:NFC:{}> **Gagnants -** :", get_nfc_emoji()));
            for team in &capsule.nfc_wins {
                content.push_str(&format!(" <:{}:{}>", team, get_team_emoji(team)));
            }
            content.push_str(&format!("  /  **Wildcards -** :"));
            for team in &capsule.nfc_wildcards {
                content.push_str(&format!(" <:{}:{}>", team, get_team_emoji(team)));
            }

            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .ephemeral(true)
                        .content(content)
                    )
            })
            .await {
                println!("![capsule] Cannot respond to slash command : {:?}", reason);
            }
        },
        Err(reason) => {
            println!("![capsule] Could not fetch capsule for season {}: {:?}", season, reason);

            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .ephemeral(true)
                        .content("Une erreur s'est produite avec la commande `/capsule`.")
                    )
            })
            .await {
                println!("![capsule] Cannot respond to slash command : {:?}", reason);
            }
        },
    }
}
