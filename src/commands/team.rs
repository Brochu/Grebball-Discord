use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::prelude::*;

use library::database::DB;
use library::football::get_team_emoji;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("equipe")
        .description("Modifier ou afficher l'équipe favorite du pooler courant")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("équipe")
                .kind(CommandOptionType::String)
                .description("La nouvelle équipe favorite")
                .required(false)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    if let Some(option) = command.data.options.first() {
        let team = option.value.as_ref().unwrap().as_str().unwrap();

        if get_team_emoji(team) != get_team_emoji("") {
            match db.update_favteam(&discordid, team).await {
                Ok(_) => { },
                Err(e) => { println!("![team] Could not update favorite team: {}", e) },
            }
        }
    }

    let (name, favteam) = match db.fetch_favteam(&discordid).await {
        Ok((name, favteam)) => { (name, favteam) }
        Err(_) => { ("Inconnu".to_string(), "NA".to_string()) }
    };
    let logo = get_team_emoji(&favteam);

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                .content(format!("Équipe favorite pour {}: <:{}:{}>", name, favteam, logo))
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
