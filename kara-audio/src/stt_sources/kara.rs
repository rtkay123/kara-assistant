use std::{
    cmp::min,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::AsyncSeekExt,
};
use tracing::{error, trace, warn};
use vosk::Recognizer;

use crate::SAMPLE_RATE;

use super::STTSource;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const VOSK_MODEL_URL: &str =
    "https://github.com/alphacep/vosk-api/releases/download/v0.3.42/vosk-linux-armv7l-0.3.42.zip";
/// Initialises a Coqui STT model with an (optional) scorer (language model).
/// Panics if the STT model could not be initialised
#[tracing::instrument]
pub(crate) async fn init_kara_model(model: &str) -> Result<STTSource> {
    use gag::Gag;
    let _print_gag = Gag::stderr().unwrap();
    trace!("initialising kara stt model");
    match vosk::Model::new(model).ok_or(format!(
        "failed to initialise kara stt model from path: {}",
        model
    )) {
        Ok(vosk_model) => {
            let mut recogniser = Recognizer::new(&vosk_model, SAMPLE_RATE as f32)
                .ok_or("failed to initialise recogniser")?;
            // recogniser.set_max_alternatives(10);
            recogniser.set_words(true);
            recogniser.set_partial_words(true);
            trace!(path = %model, "located model");
            trace!("kara stt model initialised");
            Ok(STTSource::Kara(Arc::new(Mutex::new(recogniser))))
        }
        Err(e) => {
            warn!("{e}");
            trace!("trying to get fallback");
            let mut data_dir = data_dir().await;
            drop(_print_gag);
            download_model(&Client::new(), VOSK_MODEL_URL, &mut data_dir).await
        }
    }
}

#[derive(Debug)]
struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self> {
        if buffer_size == 0 {
            Err("invalid buffer_size, give a value greater than zero.")?;
        }
        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = reqwest::header::HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(
                reqwest::header::HeaderValue::from_str(&format!(
                    "bytes={}-{}",
                    prev_start,
                    self.start - 1
                ))
                .expect("string provided by format!"),
            )
        }
    }
}

#[tracing::instrument]
async fn download_model(client: &Client, url: &str, path_buf: &mut PathBuf) -> Result<STTSource> {
    let head = client.head(url).send().await?;
    let content_length = head.headers().get(CONTENT_LENGTH);
    let accept_range = head.headers().get(ACCEPT_RANGES);
    let file_name = &head
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.zip");
    let mut outfile = OpenOptions::new();
    path_buf.push(file_name);
    let mut outfile = outfile
        .read(true)
        .append(true)
        .create(true)
        .open(&path_buf)
        .await
        .unwrap();
    let file_size = tokio::fs::metadata(path_buf).await.unwrap().len();
    outfile
        .seek(tokio::io::SeekFrom::Start(file_size))
        .await
        .unwrap();
    let mut downloaded = file_size;

    let c: u64 = content_length.unwrap().to_str().unwrap().parse().unwrap();
    let pb = ProgressBar::new(c);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(format!("Downloading {}", url));
    match content_length {
        Some(content_length) => {
            let content_length = content_length.to_str().unwrap();
            let content_length: u64 = content_length.parse().unwrap();
            match accept_range {
                Some(_) => {
                    // resume download if file exists
                    // check file size
                    for range in PartialRangeIter::new(file_size, content_length - 1, 10240)? {
                        let response = client.get(url).header(RANGE, range).send().await?;
                        let status = response.status();
                        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
                            error!("Unexpected server response: {}", status)
                        } else {
                            let content = response.bytes().await.unwrap();
                            let mut content = content.as_ref();
                            tokio::io::copy(&mut content, &mut outfile).await.unwrap();
                        }
                        let new = min(downloaded + 10240, content_length);
                        downloaded = new;
                        pb.set_position(downloaded);
                    }
                    let content = head.bytes().await.unwrap();
                    let mut content = content.as_ref();
                    tokio::io::copy(&mut content, &mut outfile).await?;
                }
                None => {
                    // redownload file
                    todo!()
                }
            }
        }
        None => {
            //redownload file
            todo!()
        }
    }

    // Make HEAD request
    todo!()
}

fn extract_file(file: File, parent: &Path, file_name: &str) -> Result<STTSource> {
    todo!()
}

async fn data_dir() -> PathBuf {
    let mut dir = dirs::data_dir().expect("could not find data dir");
    dir.push("kara");
    dir.push("stt");
    create_dir_all(&dir).await.unwrap();
    dir
}
