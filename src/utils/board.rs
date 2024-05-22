use chrono::Utc;
use poise::serenity_prelude as serenity;
use ::serenity::all::EditMessage;
use serenity::{
    Http, ChannelType, Message, GuildChannel, GuildId, CreateThread, CreateMessage, CreateEmbed, CacheHttp
};

use std::sync::Arc;

use super::youtube_dl::MetaData;

#[derive(Clone, Debug)]
pub struct Board {
    thread: Option<GuildChannel>,
    score: Option<Message>,
    status: Option<Message>,
    http: Arc<Http>
}

impl Board {
    pub fn new(http: Arc<Http>) -> Self {
        Self {
            thread: None,
            score: None,
            status: None,
            http
        }
    }

    pub async fn delete(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.delete(&self.http).await.unwrap();
            self.score = None;
            self.status = None;
        }
    }

    pub async fn set(&mut self, channel: &GuildChannel) {
        self.delete().await;

        let thread = CreateThread::new("Board").kind(ChannelType::PublicThread);
        let thread_channel = channel
            .create_thread(&self.http, thread)
            .await
            .unwrap();
        
        let score_embed = CreateEmbed::new().title("Score");
        let score = thread_channel
            .send_message(&self.http, CreateMessage::new().add_embed(score_embed))
            .await
            .unwrap();

        let status_embed = CreateEmbed::new().title("재생중인 곡");
        let status = thread_channel
            .send_message(&self.http, CreateMessage::new().add_embed(status_embed))
            .await
            .unwrap();

        self.thread = Some(thread_channel);
        self.score = Some(score);
        self.status = Some(status);
    }

    pub async fn edit(&mut self, meta: MetaData) {
        if let Some(score) = &mut self.score {
            score.edit(&self.http, 
                EditMessage::new().add_embed(CreateEmbed::new().title("Score").description(""))).await.unwrap();
        }
        let after = Utc::now().timestamp() + meta.duration.unwrap();
        if let Some(status) = &mut self.status {
            status.edit(&self.http, 
                EditMessage::new().add_embed(
                    CreateEmbed::new()
                        .title("재생중인 곡")
                        .description(format!("{}\n<t:{}:R>", meta.title.unwrap(), after))))
                .await
                .unwrap();
        }
        
    }

}