use songbird::input::{
    AudioStream,
    AudioStreamError,
    AuxMetadata,
    File,
};
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use std::{path::Path, error::Error, io::ErrorKind, collections::HashMap};
use symphonia::core::io::MediaSource;
use tokio::process::Command;

const YOUTUBE_DL_COMMAND: &str = "yt-dlp";
const FFMPEG_COMMAND: &str = "ffmpeg";
const TMP_FOLDER: &str = "./target/tmp/";
// "yt-dlp "https://www.youtube.com/watch?v=KIiO7rGnW6I&list=PLy8FIz514mc8kUd6irZ3_djzblgBmuajc&index=129" -f "ba[abr>0][vcodec=none]/best" --no-playlist --downloader ffmpeg --downloader-args ffmpeg:"-ss 30 -t 30""

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Output {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<i64>,
    pub filesize: Option<u64>,
    pub title: Option<String>,
    pub track: Option<String>,
    pub url: String,
    pub webpage_url: Option<String>,
    pub audio_ext: Option<String>,
}

pub async fn ytdl_optioned(
    url: &String, mut start: i64, mut duration: i64
) -> Result<(String, Output), AudioStreamError> {
    let mut ytdl_output = format!("{TMP_FOLDER}{url}.%(ext)s");
    let ytdl_args = [
        "-j",
        "--no-simulate",
        url,
        "-f",
        "ba[abr<=100][vcodec=none]/best",
        "--no-playlist",
        "-o",
        ytdl_output.as_str()
    ];

    let mut ytdl = Command::new(YOUTUBE_DL_COMMAND)
        .args(ytdl_args)
        .output()
        .await
        .map_err(|e| {
            AudioStreamError::Fail(if e.kind() == ErrorKind::NotFound {
                format!("could not find executable '{}' on path", YOUTUBE_DL_COMMAND).into()
            } else {
                Box::new(e)
            })
        })?;
    
    if !ytdl.status.success() {
        return Err(AudioStreamError::Fail(
            format!(
                "{} failed with non-zero status code: {}",
                YOUTUBE_DL_COMMAND,
                std::str::from_utf8(&ytdl.stderr[..]).unwrap_or("<no error message>")
            )
            .into(),
        ));
    }

    let out = ytdl
        .stdout
        .split_mut(|&b| b == b'\n')
        .filter_map(|x| (!x.is_empty()).then(|| serde_json::from_slice(x)))
        .collect::<Result<Vec<Output>, _>>()
        .map_err(|e| AudioStreamError::Fail(Box::new(e)))?;

    let mut meta = out
        .first()
        .ok_or_else(|| {
            AudioStreamError::Fail(format!("no results found for '{url}'").into())
        }).unwrap().clone();
    ytdl_output = format!("{TMP_FOLDER}{url}.{}", meta.audio_ext.as_ref().unwrap());

    if duration == 0 {
        Ok((ytdl_output, meta))
    } else {
        let ffmpeg_output = format!("{TMP_FOLDER}{url}_cut.{}", meta.audio_ext.as_ref().unwrap());
        let ffmpeg_args = [
            "-y",
            "-i",
            ytdl_output.as_str(),
            "-ss",
            &start.to_string(),
            "-t",
            &duration.to_string(),
            ffmpeg_output.as_str(),
        ];

        Command::new(FFMPEG_COMMAND)
            .args(ffmpeg_args)
            .output()
            .await
            .map_err(|e| {
                AudioStreamError::Fail(if e.kind() == ErrorKind::NotFound {
                    format!("could not find executable '{}' on path", FFMPEG_COMMAND).into()
                } else {
                    Box::new(e)
                })
            })?;
        meta.duration = Some(duration);
        Ok((ffmpeg_output, meta))
    }

}

