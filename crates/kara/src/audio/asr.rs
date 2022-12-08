use anyhow::Result;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use asr::sources::kara::LocalRecogniser;
use crossbeam_channel::Sender;
use iced_winit::winit::event_loop::EventLoopProxy;
use res_get::ResGet;
use tracing::{debug, error, info};

use crate::events::KaraEvent;

pub fn try_default_location(
    model_path: impl AsRef<Path> + std::marker::Send,
    sample_rate: f32,
) -> Result<LocalRecogniser> {
    Ok(LocalRecogniser::new(model_path, sample_rate)?)
}

pub async fn get_remote_model(
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    sender: Sender<LocalRecogniser>,
    fallback_url: impl AsRef<str>,
    model_path: impl AsRef<Path>,
    sample_rate: f32,
) -> Result<()> {
    let model_path = model_path.as_ref().to_owned();
    let res_get = ResGet::new(fallback_url.as_ref(), &model_path);
    let progress = res_get.get_progress().clone();
    tokio::spawn(async move {
        // send with sender
        tracing::warn!(
            model_path = model_path.display().to_string(),
            "trying default sender"
        );

        if let Err(e) = try_default_location(model_path, sample_rate).and_then(|model| {
            let _ = sender.send(model);
            Ok(())
        }) {
            res_get.get_asr_model().await.and_then(|| {
                if let Err(e) = try_default_location(model_path, sample_rate).and_then(|model| {
                    let _ = sender.send(model);
                    Ok(())
                }) {
                    error!("{e}");
                }
            });
        }
    });
    tokio::spawn(async move {
        while let Ok(progress) = progress.recv() {
            {
                match event_loop.lock() {
                    Ok(proxy) => {
                        let _ = proxy.send_event(KaraEvent::UpdateProgressBar(progress));
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
        }
        debug!("exiting receiver loop");
    });
    Ok(())
}
