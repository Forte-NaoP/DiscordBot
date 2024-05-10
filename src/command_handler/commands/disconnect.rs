use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption
};

use crate::{command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, connection_handler::terminate_connection};

struct DisConnect;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(DisConnect)
}

#[async_trait]
impl CommandInterface for DisConnect {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &CommandInteraction, 
        _options: &[CommandDataOption]
    ) -> CommandReturn {
        terminate_connection(ctx, command).await;
        CommandReturn::String("접속 종료".to_owned())
    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("leave")
            .description("leave the voice channel")
    }
}