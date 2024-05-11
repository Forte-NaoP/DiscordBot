use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, client::Context, http::Http, model::id::ChannelId, prelude::RwLock
};
use songbird::{
    Call, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};

pub struct TrackEndNotifier;

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}