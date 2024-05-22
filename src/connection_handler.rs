use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, VoiceState,
    UserId
};

use songbird::{error::JoinError, tracks::TrackQueue, TrackEvent};
use tokio::time::error::Elapsed;

use std::{collections::HashMap, sync::Arc};

use crate::{event_handler::track_event_handler::TrackEndNotifier, utils::guild_queue::GuildQueue, GuildQueueKey};

#[derive(Debug)]
pub enum ConnectionErrorCode {
    AlreadyInUse,
    JoinVoiceChannelFirst,
    VoiceChannelNotFound,
    ServerNotFound,
    JoinError(JoinError)
}

pub enum ConnectionSuccessCode {
    AlreadyConnected,
    NewConnection,
}

pub async fn establish_connection(ctx: &Context, command: &CommandInteraction) -> Result<ConnectionSuccessCode, ConnectionErrorCode> {
    let guild_id = command.guild_id.unwrap();

    let user_channel = {
        // CacheRef is not Send
        let voice_states: &HashMap<UserId, VoiceState> = &guild_id
            .to_guild_cached(ctx)
            .unwrap()
            .voice_states;
        voice_states
            .get(&command.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }; // So must drop here
    
    if let Some(user_channel) = user_channel {
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if let Some(call) = manager.get(guild_id) {
            let mut call = call.lock().await;
            if let Some(bot_channel) = call.current_channel() {
                if bot_channel == user_channel.into() {
                    Ok(ConnectionSuccessCode::AlreadyConnected)
                } else {
                    Err(ConnectionErrorCode::AlreadyInUse)
                }
            } else {
                match call.join(user_channel).await {
                    Ok(_) => Ok(ConnectionSuccessCode::NewConnection),
                    Err(why) => Err(ConnectionErrorCode::JoinError(why))
                }
            }
        } else {
            match manager.join(guild_id, user_channel).await {
                Ok(_/*handler_lock*/) => {
                    // let mut handler = handler_lock.lock().await;
                    // handler.add_global_event(TrackEvent::End.into(), TrackEndNotifier);
                    Ok(ConnectionSuccessCode::NewConnection)
                },
                Err(why) => Err(ConnectionErrorCode::JoinError(why))
            }
        }
    } else {
        Err(ConnectionErrorCode::JoinVoiceChannelFirst)
    }
}

pub async fn terminate_connection(ctx: &Context, command: &CommandInteraction) {
    let guild_id = command.guild_id.unwrap();
    let voice_manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = voice_manager.get(guild_id).expect("Guild Not Found");
    handler_lock
        .lock()
        .await
        .leave()
        .await
        .expect("Disconnect Fail");
}