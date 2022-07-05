use std::{
    cmp::min,
    fs::{self, File},
    io::{Seek, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Context;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tracing::{trace, warn};
use vosk::Recognizer;

use crate::SAMPLE_RATE;

use super::STTSource;

const VOSK_MODEL_URL: &str = "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip";
/// Initialises a Coqui STT model with an (optional) scorer (language model).
/// Panics if the STT model could not be initialised
#[tracing::instrument]
pub(crate) async fn init_kara_model(model: &str) -> anyhow::Result<STTSource> {
    use gag::Gag;
    let _print_gag = Gag::stderr().unwrap();
    trace!("initialising kara stt model");
    match vosk::Model::new(model).context(format!(
        "failed to initialise kara stt model from path: {}",
        model
    )) {
        Ok(vosk_model) => {
            let mut recogniser = Recognizer::new(&vosk_model, SAMPLE_RATE as f32)
                .context("failed to initialise recogniser")?;
            // recogniser.set_max_alternatives(10);
            recogniser.set_words(true);
            recogniser.set_partial_words(true);
            trace!(path = %model, "located model");
            trace!("kara stt model initialised");
            Ok(STTSource::Kara(Arc::new(Mutex::new(recogniser))))
        }
        Err(e) => {
            warn!("{}, getting fallback...", e);
            let mut data_dir = data_dir();
            drop(_print_gag);
            println!("Oops, it looks like you haven't pointed to where you speech to text model is in my config.");
            println!("I'm going to try to find a fallback...");
            download_model(&Client::new(), VOSK_MODEL_URL, &mut data_dir).await
        }
    }
}

#[tracing::instrument]
async fn download_model(
    client: &Client,
    url: &str,
    path_buf: &mut PathBuf,
) -> anyhow::Result<STTSource> {
    let res = client
        .get(url)
        .send()
        .await
        .context(format!("Failed to GET from '{}'", &url))?;
    let total_size = res
        .content_length()
        .context(format!("Failed to get content length from '{}'", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(format!("Downloading {}", url));

    let mut file;
    let mut downloaded: u64 = 0;
    let file_name = &res
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.zip");
    let file_name = file_name.to_string();
    path_buf.push(&file_name);
    let path = path_buf.display().to_string();
    let mut stream = res.bytes_stream();

    trace!("Seeking in file. {}", path);
    if std::path::Path::new(&path).exists() {
        println!("File exists at {}. Resuming", path);
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&path)
            .unwrap();

        let file_size = std::fs::metadata(&path).unwrap().len();
        file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
        downloaded = file_size;
    } else {
        trace!(path=?file_name, "Creating new model file... ");
        // file = File::create(&path).context(format!("Failed to create file '{}'", path))?;
        file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .context("failed to create the file")?;
    }
    println!("Starting download transfer");
    while let Some(item) = stream.next().await {
        let chunk = item.context("Error while downloading file".to_string())?;
        file.write(&chunk)
            .context("Error while writing to file".to_string())?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path));

    let file_name = PathBuf::from(file_name);
    let file_name = file_name.file_name();
    let file_name = file_name.unwrap().to_string_lossy().to_string();
    extract_file(file, path_buf.parent().unwrap(), &file_name)
}

fn extract_file(file: File, parent: &Path, file_name: &str) -> Result<STTSource, anyhow::Error> {
    let mut archive = zip::ZipArchive::new(file)?;

    std::fs::create_dir_all(parent).unwrap();

    archive.extract(parent).unwrap();

    let vosk_model =
        vosk::Model::new(format!("{}/{file_name}", parent.display())).context(format!(
            "failed to initialise kara stt model from path: {:?}",
            parent
        ))?;

    let mut recogniser = Recognizer::new(&vosk_model, SAMPLE_RATE as f32)
        .context("failed to initialise recogniser")?;
    // recogniser.set_max_alternatives(10);
    recogniser.set_words(true);
    recogniser.set_partial_words(true);
    trace!("kara stt model initialised");
    Ok(STTSource::Kara(Arc::new(Mutex::new(recogniser))))
}

fn data_dir() -> PathBuf {
    let mut dir = dirs::data_dir().expect("could not find data dir");
    dir.push("kara");
    dir.push("stt");
    fs::create_dir_all(&dir).expect("could not create data dir");
    dir
}
