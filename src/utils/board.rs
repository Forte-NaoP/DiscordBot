use chrono::Utc;
use poise::serenity_prelude as serenity;
use ::serenity::all::EditMessage;
use serenity::{
    Http, ChannelType, Message, GuildChannel, GuildId, CreateThread, CreateMessage, CreateEmbed, CacheHttp
};

use std::mem;
use std::sync::Arc;
use std::collections::HashMap;

use super::youtube_dl::MetaData;

#[derive(Clone, Debug)]
pub struct Board {
    thread: Option<GuildChannel>,
    score_message: Option<Message>,
    status_message: Option<Message>,
    score: HashMap<String, i32>,
    http: Arc<Http>
}

impl Board {
    pub fn new(http: Arc<Http>) -> Self {
        Self {
            thread: None,
            score_message: None,
            status_message: None,
            score: HashMap::new(),
            http
        }
    }

    pub async fn delete(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.delete(&self.http).await.unwrap();
            self.score_message = None;
            self.status_message = None;
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
        self.score_message = Some(score);
        self.status_message = Some(status);
    }

    pub async fn edit_score(&mut self) {
        if let Some(score) = &mut self.score_message {
            let mut score_vec: Vec<(i32, String)> = self.score.iter().map(|(k, &v)| (v, k.clone())).collect();
            score_vec.sort_by(|a, b| b.0.cmp(&a.0));
            let score_text = score_vec.iter().map(|(v, k)| format!("{}: {}", k, v)).collect::<Vec<String>>().join("\n");
            score.edit(&self.http, 
                EditMessage::new().add_embed(CreateEmbed::new().title("Score").description(score_text))).await.unwrap();
        }
    }

    pub async fn edit_status(&mut self, meta: Option<MetaData>) {
        if let Some(status) = &mut self.status_message {
            let embed = match meta {
                Some(meta) => {
                    let after = Utc::now().timestamp() + meta.duration.unwrap();
                    CreateEmbed::new()
                        .title("재생중인 곡")
                        .description(format!("{}\n<t:{}:R> 종료", meta.title.unwrap(), after))
                }, 
                None => {
                    CreateEmbed::new().title("재생중인 곡").description("없음")
                }
            };
            status.edit(&self.http, EditMessage::new().add_embed(embed))
                .await
                .unwrap();
        }
    }

    pub async fn add_user(&mut self, member: String) {
        if self.score.contains_key(&member) {
            return;
        }
        self.score.insert(member, 0);
        self.edit_score().await;
    }

    pub async fn add_users(&mut self, members: Vec<String>) {
        for member in members {
            if self.score.contains_key(&member) {
                continue;
            }
            self.score.insert(member, 0);
        }
        self.edit_score().await;
    }

    pub async fn add_score(&mut self, member: String) {
        if let Some(value) = self.score.get_mut(&member) {
            *value += 1;
            self.edit_score().await;
        }
    }

    pub async fn reset_score(&mut self) {
        self.score.clear();
        self.edit_score().await;
    }

}