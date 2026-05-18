use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::{DB, CapsuleStatus};
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
    let poolid = env::var("POOL_ID")
        .expect("![capsule] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![capsule] Could not parse pool_id to int");

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    if let Ok(status) = db.prime_capsule(&discordid, &season, &poolid).await {
        match status {
            CapsuleStatus::Primed => {
                let picks_url = env::var("PICKS_URL")
                    .expect("![capsule] Could not find 'PICKS_URL' env var");
                let url = format!("{}/capsule/{}/{}", picks_url, discordid, season);

                if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                    res
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m
                            .ephemeral(true)
                            .content(format!("Prêt pour les prédictions de la capsule {} à faire ici: {}", season, url))
                        )
                })
                .await {
                    println!("![capsule] Cannot respond to slash command : {:?}", reason);
                }
            },
            CapsuleStatus::Filled(capsule) => {
                let mut content = String::new();
                content.push_str(format!("**Capsule {}**\n\n", season).as_str());

                content.push_str(format!("**Gagnants -** <:{}:{}> : ", "AFC", get_afc_emoji()).as_str());
                let afc_wins = &capsule.afc_winners.unwrap_or("".to_owned());
                for winner in afc_wins.split(",") {
                    content.push_str(&format!("  <:{}:{}>", winner, get_team_emoji(winner)));
                }

                content.push_str(format!(" / **Wildcards -** <:{}:{}> : ", "AFC", get_afc_emoji()).as_str());
                let afc_wild = &capsule.afc_wildcards.unwrap_or("".to_owned());
                for wild in afc_wild.split(",") {
                    content.push_str(&format!("  <:{}:{}>", wild, get_team_emoji(wild)));
                }

                content.push_str(format!("\n**Gagnants -** <:{}:{}> : ", "NFC", get_nfc_emoji()).as_str());
                let nfc_wins = &capsule.nfc_winners.unwrap_or("".to_owned());
                for winner in nfc_wins.split(",") {
                    content.push_str(&format!("  <:{}:{}>", winner, get_team_emoji(winner)));
                }

                content.push_str(format!("  /  **Wildcards -** <:{}:{}> : ", "NFC", get_nfc_emoji()).as_str());
                let nfc_wild = &capsule.nfc_wildcards.unwrap_or("".to_owned());
                for wild in nfc_wild.split(",") {
                    content.push_str(&format!("  <:{}:{}>", wild, get_team_emoji(wild)));
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
        }
    } else {
        println!("[capsule] Error while priming capsule for season {}", season);

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
    }
}
