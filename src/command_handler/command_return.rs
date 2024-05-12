use poise::serenity_prelude as serenity;
use serenity::{
    CreateEmbed, Context, CommandInteraction, async_trait};
use songbird::tracks::TrackHandle;

use std::any::Any;
use std::sync::Arc;

use crate::utils::youtube_dl::Output;

pub enum CommandReturn {
    String(String),
    SingleEmbed(CreateEmbed),
    SongInfoEmbed(TrackHandle, Output),
    ControlInteraction(Box<dyn ControlInteraction + Send + Sync>),
    None,
}

#[async_trait]
pub trait ControlInteraction {
    async fn control_interaction(
        &mut self,
        ctx: Arc<Context>, 
        interaction: CommandInteraction, 
    ) -> Result<(), serenity::Error>;
    fn as_any(&self) -> &dyn Any;
}

