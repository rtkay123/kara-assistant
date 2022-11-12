use std::{
    fmt::Debug,
    path::Path,
    sync::{Arc, Mutex},
};

#[cfg(feature = "graphical")]
use iced_winit::winit::event_loop::EventLoopProxy;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{error, trace, warn};

use crate::events::KaraEvent;

use super::read_file;

#[cfg(feature = "graphical")]
pub fn monitor_config(event_loop_proxy: Arc<Mutex<EventLoopProxy<KaraEvent>>>) {
    if let Some(path) = dirs::config_dir() {
        tokio::task::spawn_blocking(move || {
            if let Err(e) = async_watch(path, event_loop_proxy) {
                error!("error: {:?}", e)
            }
        });
    }
}

fn async_watcher() -> notify::Result<(
    RecommendedWatcher,
    crossbeam_channel::Receiver<notify::Result<Event>>,
)> {
    let (tx, rx) = crossbeam_channel::unbounded();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            tx.send(res).unwrap();
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

#[cfg(feature = "graphical")]
#[tracing::instrument]
fn async_watch(
    path: impl AsRef<Path> + Debug,
    event_loop_proxy: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
) -> notify::Result<()> {
    let (mut watcher, rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Ok(res) = rx.recv() {
        match res {
            Ok(event) => {
                event.paths.iter().any(|f| {
                    if let Some(file_name) = f.file_name() {
                        if file_name == "kara.toml" {
                            let mut current_path = path.as_ref().to_path_buf();
                            current_path.push("kara.toml");

                            match read_file(&current_path) {
                                Ok(Ok(config)) => {
                                    let proxy = event_loop_proxy.lock().unwrap();
                                    if let Err(e) =
                                        proxy.send_event(KaraEvent::ReloadConfiguration(config))
                                    {
                                        error!("send event error {:?}", e);
                                    } else {
                                        trace!("configuration reloaded");
                                    }
                                }
                                Ok(Err(e)) => {
                                    error!("{e}");
                                }
                                Err(e) => {
                                    error!("{e}");
                                }
                            }
                        }
                        true
                    } else {
                        false
                    }
                });
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
    Ok(())
}
