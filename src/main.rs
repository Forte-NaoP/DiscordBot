use std::sync::Arc;

use serenity::{all::GuildId, FutureExt};
use songbird::{tracks::TrackQueue, typemap::{TypeMap, TypeMapKey}, SerenityInit};
use tokio::signal::ctrl_c;

use poise::serenity_prelude as serenity_poise;
use serenity::model::prelude::GatewayIntents;

use reqwest::Client as HttpClient;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, FmtSubscriber};
use dashmap::DashMap;
use utils::guild_queue::GuildQueue;

mod event_handler;
mod command_handler;
mod connection_handler;
mod utils;
mod global;

struct GuildQueueKey;
impl TypeMapKey for GuildQueueKey {
    type Value = Arc<DashMap<GuildId, GuildQueue>>;
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("info"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGES
    ;

    let mut client: serenity::prelude::Client = serenity_poise::ClientBuilder::new(token, intents)
        .event_handler(event_handler::event_handler::DiscordEventHandler)
        .type_map_insert::<GuildQueueKey>(Arc::new(DashMap::new()))
        .register_songbird()
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();
    let client_future = client.start_autosharded().fuse();

    tokio::pin!(client_future);

    let ctrlc = ctrl_c().fuse();
    tokio::pin!(ctrlc);

    tokio::select! {
        client_result = client_future => {
            if let Err(why) = client_result {
                println!("An error occurred while running the client: {:?}", why);
            }
        },
        _ = ctrlc => {
            println!("Ctrl-C received, shutting down");
            shard_manager.shutdown_all().await;
        }
    };
}
