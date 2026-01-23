use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::{DB, CapsuleStatus};
use library::football::get_team_emoji;

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

    if let Ok(status) = db.prime_capsule(&discordid, &season).await {
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
                let mut content = format!("**Capsule {}**\n\n", season);

                // AFC Division Winners
                content.push_str("**Gagnants de divisions - AFC:**\n");
                if let Some(team) = &capsule.winafcn {
                    content.push_str(&format!("  Nord: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winafcs {
                    content.push_str(&format!("  Sud: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winafce {
                    content.push_str(&format!("  Est: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winafcw {
                    content.push_str(&format!("  Ouest: <:{}:{}>\n", team, get_team_emoji(team)));
                }

                // NFC Division Winners
                content.push_str("\n**Gagnants de divisions - NFC:**\n");
                if let Some(team) = &capsule.winnfcn {
                    content.push_str(&format!("  Nord: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winnfcs {
                    content.push_str(&format!("  Sud: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winnfce {
                    content.push_str(&format!("  Est: <:{}:{}>\n", team, get_team_emoji(team)));
                }
                if let Some(team) = &capsule.winnfcw {
                    content.push_str(&format!("  Ouest: <:{}:{}>\n", team, get_team_emoji(team)));
                }

                // Wildcards
                if let Some(wildcards) = &capsule.afcwildcards {
                    let (first, rest) = wildcards.split_once(",").unwrap();
                    let (second, third) = rest.split_once(",").unwrap();

                    content.push_str(&format!("\n**Wildcards - AFC:** <:{}:{}> <:{}:{}> <:{}:{}>\n",
                        first, get_team_emoji(first),
                        second, get_team_emoji(second),
                        third, get_team_emoji(third),
                    ));
                }
                if let Some(wildcards) = &capsule.nfcwildcards {
                    let (first, rest) = wildcards.split_once(",").unwrap();
                    let (second, third) = rest.split_once(",").unwrap();

                    content.push_str(&format!("\n**Wildcards - NFC:** <:{}:{}> <:{}:{}> <:{}:{}>\n",
                        first, get_team_emoji(first),
                        second, get_team_emoji(second),
                        third, get_team_emoji(third),
                    ));
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
