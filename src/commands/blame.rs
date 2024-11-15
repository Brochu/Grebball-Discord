use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("blame")
        .description("Blamer une équipe - Calcule la difference de points gagné/perdu par une équipe")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("équipe")
                .kind(CommandOptionType::String)
                .description("L'équipe à blamer")
                .required(true)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, _db: &DB) {
    let _season = env::var("CONF_SEASON")
        .expect("[picks] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[picks] Could not parse 'CONF_SEASON' to u16");

    let _discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let team = command.data.options.first().unwrap().clone().value.unwrap();
    let team = team.as_str().unwrap();

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                .content(format!("`/blame` was called with team = `{}`.", team))
            )
    })
    .await {
        println!("![picks] Cannot respond to slash command : {:?}", reason);
    }
}
