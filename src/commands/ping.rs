use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn _run(_options: &[CommandDataOption]) -> String {
    return "".to_string();
}

pub fn _register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping")
        .description("A ping command")
}
