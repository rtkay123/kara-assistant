use std::{
    cmp::min,
    path::{Path, PathBuf},
};

use futures_util::StreamExt;
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tracing::{error, trace};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct VoskModel {
    url: String,
    destination: PathBuf,
    progress: crossbeam_channel::Receiver<f32>,
}

pub struct ResGet {
    client: Client,
    vosk_model: VoskModel,
    progress_sender: crossbeam_channel::Sender<f32>,
}

impl ResGet {
    pub fn new(url: &str, destination: impl AsRef<Path>) -> Self {
        let client = Client::new();
        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            client,
            vosk_model: VoskModel {
                url: url.to_string(),
                destination: destination.as_ref().to_path_buf(),
                progress: rx,
            },
            progress_sender: tx,
        }
    }

    pub fn get_progress(&self) -> &crossbeam_channel::Receiver<f32> {
        &self.vosk_model.progress
    }

    pub async fn get_asr_model(&self) -> Result<()> {
        create_dir_all(&self.vosk_model.destination).await?;
        let mut path_buf = self.vosk_model.destination.clone();
        let head = self.client.head(&self.vosk_model.url).send().await?;
        let content_length = head.headers().get(CONTENT_LENGTH);
        let accept_range = head.headers().get(ACCEPT_RANGES);
        let n_url = head.url();

        let file_name = n_url
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
            .await?;

        let file_size = tokio::fs::metadata(&path_buf).await?.len();

        outfile.seek(tokio::io::SeekFrom::Start(file_size)).await?;
        let mut downloaded = file_size;

        let url = &self.vosk_model.url;

        match content_length {
            Some(content_length) => {
                let content_length = content_length.to_str()?;
                let content_length: u64 = content_length.parse()?;
                if accept_range.is_some() {
                    // resume download if file exists
                    // check file size
                    for range in PartialRangeIter::new(file_size, content_length - 1, 10240)? {
                        let response = self.client.get(url).header(RANGE, range).send().await?;
                        let status = response.status();
                        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
                            error!("Unexpected server response: {status}");
                        } else {
                            let content = response.bytes().await?;
                            let mut content = content.as_ref();
                            tokio::io::copy(&mut content, &mut outfile).await?;
                        }
                        let new = min(downloaded + 10240, content_length);
                        downloaded = new;
                        let progress = downloaded as f32 / content_length as f32 * 100.0;
                        self.progress_sender.send(progress)?;
                    }
                    let file = File::open(&path_buf).await?;
                    let content = head.bytes().await?;
                    let mut content = content.as_ref();
                    tokio::io::copy(&mut content, &mut outfile).await?;
                    extract_file(
                        file,
                        path_buf.parent().ok_or("no parent")?,
                        &path_buf.display().to_string(),
                    )
                    .await?;
                } else {
                    // redownload file
                    download_no_resume(
                        &self.client,
                        &path_buf,
                        url,
                        &file_name,
                        &self.progress_sender,
                    )
                    .await?;
                }
            }
            None => {
                //redownload file
                download_no_resume(
                    &self.client,
                    &path_buf,
                    url,
                    &file_name,
                    &self.progress_sender,
                )
                .await?;
            }
        }

        Ok(())
    }
}

async fn download_no_resume(
    client: &Client,
    path_buf: &Path,
    url: &str,
    file_name: &str,
    progress_sender: &crossbeam_channel::Sender<f32>,
) -> Result<()> {
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|_| format!("Failed to GET from '{}'", &url))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{url}'"))?;

    let mut file = File::create(path_buf).await?;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let progress = downloaded as f32 / total_size as f32 * 100.0;
        progress_sender.send(progress)?;
    }
    extract_file(file, path_buf.parent().ok_or("no parent")?, file_name).await?;
    Ok(())
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

async fn extract_file(file: File, parent: &Path, file_name: &str) -> Result<()> {
    use gag::Gag;
    let file = file.into_std().await;
    let _print_gag = Gag::stderr()?;
    let file_name = PathBuf::from(file_name);
    let file_name = file_name.file_name().ok_or("could not get file name")?;
    let file_name = file_name.to_string_lossy().to_string();
    let parent_str = parent.to_string_lossy().to_string();
    trace!(
        file_name = file_name,
        parent_dir = parent_str,
        "extracting file"
    );
    let mut archive = zip::ZipArchive::new(file)?;

    tokio::fs::create_dir_all(parent).await?;

    archive.extract(parent)?;
    Ok(())
}
