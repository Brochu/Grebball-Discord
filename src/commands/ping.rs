use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ping")
        .description("A ping command")
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    println!("[ping] User: {:?}", command.user);
    println!("[ping] Avatar: {:?}", command.user.avatar);
    println!("[ping] Avatar URL: {:?}", command.user.avatar_url());

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Ping response!")
            )
    })
    .await {
        println!("![ping] Cannot respond to slash command : {:?}", reason);
    }
}
