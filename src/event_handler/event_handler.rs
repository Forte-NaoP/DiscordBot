use poise::serenity_prelude::{
    async_trait, Command, Interaction, Context, Ready, EventHandler, Message, VoiceState
};
use serenity::{all::CacheHttp, model::{channel, guild}};
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
        
        if let Some(channel) = guild_queue.channel()
            .filter(|channel| *channel == msg.channel_id.into())
        {
            if let Some((_, Some(keyword))) = guild_queue.current()
                .map(|(_, meta)| ((), meta.keyword))
                .filter(|(_, kw)| kw.as_ref().map_or(false, |kw| kw == msg.content.trim()))
            {

                let nickname = msg.member(ctx.http.clone()).await.unwrap().display_name().to_owned();
                guild_queue.get_board().lock().await.add_score(nickname).await;
                guild_queue.skip().unwrap();
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let guild_id = new.guild_id.unwrap();
        let cache = ctx.cache().unwrap().clone();
        let http = ctx.http.clone();
        let guild_name = guild_id.name(cache.clone()).unwrap();
        
        let (nickname, old_channel_id) = match old {
            Some(old) => {
                let nickname = old.member.unwrap().display_name().to_owned();
                let channel_name = old.channel_id.unwrap().name(http.clone()).await.unwrap();
                println!("user {} in {} of {} before", nickname, channel_name, guild_name);
                (Some(nickname), old.channel_id)
            },
            None => (None, None)
        };

        let new_channel_id = if let Some(channel_id) = new.channel_id {
            let nickname = nickname.unwrap_or(new.member.unwrap().display_name().to_owned());
            let channel_name = channel_id.name(http.clone()).await.unwrap();

            println!("user {} join in {} of {} after",
                nickname,
                channel_name,
                guild_name
            );

            let members = channel_id
                .to_channel_cached(&cache)
                .unwrap()
                .members(cache.clone())
                .unwrap()
                .iter()
                .map(|member| member.display_name().to_owned())
                .collect::<Vec<String>>();

            let guild_queue = get_guild_queue(&ctx, guild_id).await;
            let board_lock = guild_queue.get_board();
            let manager = songbird::get(&ctx).await.unwrap().clone();
            if let Some(handler_lock) = manager.get(guild_id) {
                let bot_voice = handler_lock.lock().await.current_channel().unwrap();
                if bot_voice == channel_id.into() {
                    let mut board = board_lock.lock().await;
                    board.add_users(members).await;
                }
            }

            Some(channel_id)

        } else {
            let nickname = nickname.unwrap_or(new.member.unwrap().display_name().to_owned());
            let channel_name = old_channel_id.unwrap().name(http.clone()).await.unwrap();

            println!("user {} leave from {} of {} after",
                nickname,
                channel_name,
                guild_name
            );
            None
        };
        
        // if let Some(old_channel_id) = old_channel_id {
        //     println!("voice channel {} in : {:?}", 
        //         old_channel_id.name(http.clone()).await.unwrap(),
        //         old_channel_id
        //             .to_channel_cached(&cache)
        //             .unwrap()
        //             .members(cache.clone())
        //             .unwrap()
        //             .iter()
        //             .map(|member| member.display_name())
        //             .collect::<Vec<&str>>()
        //     );
        // }

        // if let Some(new_channel_id) = new_channel_id {
        //     println!("voice channel {} in : {:?}", 
        //         new_channel_id.name(http.clone()).await.unwrap(),
        //         new_channel_id
        //             .to_channel_cached(&cache)
        //             .unwrap()
        //             .members(cache.clone())
        //             .unwrap()
        //             .iter()
        //             .map(|member| member.display_name())
        //             .collect::<Vec<&str>>()
        //     );
        // }
    }
}