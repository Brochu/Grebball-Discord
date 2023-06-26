use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
//use library::football::calc_results;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("season")
        .description("Request the results all current pool members for the current season")
        .kind(CommandType::ChatInput)
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let _season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let _poolid = db.find_poolid(&discordid).await
        .expect("![results] Could not find valid poolid based on discordid");

    let _weeks = vec!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 160, 125, 150, 200);

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Here are the results for the current season ...")
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
