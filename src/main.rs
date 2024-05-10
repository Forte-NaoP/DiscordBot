use serenity::FutureExt;
use songbird::SerenityInit;
use tokio::signal::ctrl_c;

use poise::serenity_prelude as serenity_poise;
use serenity::model::prelude::GatewayIntents;

mod event_handler;
mod command_handler;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
pub struct Data {}

#[tokio::main]
async fn main() {

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

    let mut client = serenity_poise::ClientBuilder::new(token, intents)
        .event_handler(event_handler::event_handler::DiscordEventHandler)
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
