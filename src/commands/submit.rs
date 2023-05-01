use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
    return "Hey, this bot is online!".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("submit")
        .kind(CommandType::Message)
}
