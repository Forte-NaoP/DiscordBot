use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption
};

use crate::command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    };

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
        let guild_id = command.guild_id.unwrap();
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        let handler_lock = manager.get(guild_id).expect("Guild Not Found");
        handler_lock
            .lock()
            .await
            .leave()
            .await
            .expect("Disconnect Fail");
        CommandReturn::String("접속 종료".to_owned())
    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("leave")
            .description("leave the voice channel")
    }
}