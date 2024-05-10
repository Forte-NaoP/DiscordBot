use poise::serenity_prelude::{
    Command, Interaction, Context, Ready, EventHandler, async_trait
};
use crate::command_handler::{
    command_handler::*,
    commands::*,
};

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());

        let commands = Command::get_global_commands(&ctx.http).await.unwrap();

        match commands.iter().find(|command| {
            "launch" == command.name.as_str()
        }) {
            Some(_) => (),
            None => {
                ctx.http.create_global_command(&launch::register()).await.unwrap();
            }
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => match command.data.name.as_str() {
                "launch" => launch::run(&ctx, command).await,
                _ => execute_command(&ctx, command).await,
            },
            _ => {},
        };
    }
}