use poise::serenity_prelude::{
    async_trait, Command, Interaction, Context, Ready, EventHandler, Message
};
use crate::{command_handler::{
    command_handler::*,
    commands::*,
}, utils::guild_queue::get_guild_queue, GuildQueueKey};

use std::sync::Arc;

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
                _ => execute_command(Arc::new(ctx), command).await,
            },
            _ => {},
        };
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let guild_id = msg.guild_id.unwrap();
        let guild_queue = get_guild_queue(&ctx, guild_id).await;
        
        if let Some(channel) = guild_queue.channel() {
            if channel == msg.channel_id.into() {
                if let Some((_, Some(word))) = guild_queue.current() {
                    if word == msg.content.trim() {
                        guild_queue.skip().unwrap();
                    }
                }
            }
        }
    }
}