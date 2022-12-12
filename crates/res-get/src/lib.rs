#[cfg(test)]
mod tests;

use std::{
    cmp::min,
    path::{Path, PathBuf},
};

use futures_util::StreamExt;
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use res_def::model_path;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tracing::{debug, error, info, trace};
use zip::ZipArchive;

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
        let destination = if destination.as_ref().as_os_str().is_empty() {
            model_path()
        } else {
            destination.as_ref().to_path_buf()
        };
        let client = Client::new();
        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            client,
            vosk_model: VoskModel {
                url: url.to_string(),
                destination,
                progress: rx,
            },
            progress_sender: tx,
        }
    }

    pub fn get_progress(&self) -> &crossbeam_channel::Receiver<f32> {
        &self.vosk_model.progress
    }

    pub async fn get_asr_model(&self) -> Result<()> {
        //     let buffer_size = 10240;
        trace!("starting model download");
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

        path_buf.push(file_name);

        let mut outfile = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path_buf)
            .await?;

        let file_size = tokio::fs::metadata(&path_buf).await?.len();

        outfile.seek(tokio::io::SeekFrom::Start(file_size)).await?;
        let mut downloaded = file_size;

        let url = self.vosk_model.url.as_str();

        match content_length {
            Some(content_length) => {
                let content_length = content_length.to_str()?;
                let content_length: u64 = content_length.parse()?;
                let buffer_size = content_length / 100;
                if accept_range.is_some() {
                    // resume download if file exists
                    // check file size
                    for range in PartialRangeIter::new(file_size, content_length - 1, buffer_size)?
                    {
                        let response = self.client.get(url).header(RANGE, range).send().await?;
                        let status = response.status();
                        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
                            error!("Unexpected server response: {status}");
                        } else {
                            let content = response.bytes().await?;
                            let mut content = content.as_ref();
                            tokio::io::copy(&mut content, &mut outfile).await?;
                        }
                        let new = min(downloaded + buffer_size as u64, content_length);
                        downloaded = new;
                        let progress = downloaded as f32 / content_length as f32 * 100.0;
                        self.progress_sender.send(progress)?;
                        debug!(download_progress = format!("{progress}%"));
                    }
                    let file = File::open(&path_buf).await?;
                    let content = head.bytes().await?;
                    let mut content = content.as_ref();
                    tokio::io::copy(&mut content, &mut outfile).await?;
                    debug!("model downloaded successfully");
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
                        file_name,
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
                    file_name,
                    &self.progress_sender,
                )
                .await?;
            }
        }
        notify_rust::Notification::new()
            .summary("Kara")
            .body("Your model is ready")
            .show()?;
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
    trace!(file_name = file_name, "starting no resume download");
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|_| format!("Failed to GET from '{}'", &url))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{url}'"))?;

    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&path_buf)
        .await?;
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

    debug!("download completed successfully");

    extract_file(
        file,
        path_buf.parent().ok_or("no parent")?,
        &path_buf.display().to_string(),
    )
    .await?;

    Ok(())
}

#[derive(Debug)]
struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u64) -> Result<Self> {
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

async fn extract_file(file: File, base_parent: &Path, file_name: &str) -> Result<()> {
    trace!(file = file_name, "attempting extraction");
    use gag::Gag;
    let _err_gag = Gag::stderr()?;
    let _print_gag = Gag::stdout()?;
    trace!("converting to std");
    let file = file.into_std().await;

    trace!("reading archive");
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        println!("loop");

        let mut file = archive.by_index(i)?;
        let mut outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        trace!("found outpath: {}", outpath.display());

        if let Some(parent) = outpath.parent() {
            if !parent.display().to_string().is_empty() {
                let mut b: Vec<_> = outpath.components().collect();
                let html = std::ffi::OsString::from(base_parent);
                b[0] = std::path::Component::Normal(&html);
                outpath = b.iter().collect();
            } else {
                outpath = base_parent.to_path_buf();
            }
        }
        trace!("modified outpath: {}", outpath.display());

        {
            let comment = file.comment();
            if !comment.is_empty() {
                debug!(file = i, comment = comment);
            }
        }

        if (*file.name()).ends_with('/') {
            debug!("dir {} extracted to \"{}\"", i, outpath.display());
            // create asr dir and pass it as parameter
            std::fs::create_dir_all(&outpath)?;
        } else {
            debug!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                let _ = std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode));
            }
        }
    }

    trace!("cleaning up artifacts");
    tokio::fs::remove_file(file_name).await?;

    let path = base_parent.display().to_string();

    info!(path = path, "local recogniser model is ready");
    Ok(())
}
