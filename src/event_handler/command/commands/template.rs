use crate::response_helper::ResponseHelper;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, args: Vec<String>) {
    ResponseHelper::new(usr, msg).push("Hello!").say().await;
}

pub async fn interaction_cmd(command: serenity::all::CommandInteraction) -> String {
    // Implementation for interaction command
    "Hello from interaction command!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("numberinput")
        .description("Test command for number input")
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::Integer,
                "int",
                "An integer from 5 to 10",
            )
            .min_int_value(5)
            .max_int_value(10)
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Number,
                "number",
                "A float from -3.3 to 234.5",
            )
            .min_number_value(-3.3)
            .max_number_value(234.5)
            .required(true),
        )
}
