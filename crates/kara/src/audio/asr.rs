use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use asr::Transcibe;
use crossbeam_channel::Sender;
use iced_winit::winit::event_loop::EventLoopProxy;
use res_get::ResGet;
use tracing::error;

use crate::events::KaraEvent;

pub async fn get_remote_model(
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    sender: Sender<Box<dyn Transcibe>>,
) -> anyhow::Result<()> {
    let res_get = ResGet::new("model-url", "model-path");
    let progress = res_get.get_progress().clone();
    tokio::spawn(async move {
        if let Err(e) = res_get.get_asr_model().await {
            error!("{e}");
        }
    });
    tokio::spawn(async move {
        while let Ok(progress) = progress.recv() {
            {
                let loops = event_loop.lock().unwrap();
                loops
                    .send_event(KaraEvent::UpdateProgressBar(progress))
                    .unwrap();
            }
        }
    });
    Ok(())
}
