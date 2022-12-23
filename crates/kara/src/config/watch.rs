use std::{
    fmt::Debug,
    path::Path,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[cfg(feature = "graphical")]
use iced_winit::winit::event_loop::EventLoopProxy;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{error, trace, warn};

use crate::events::KaraEvent;

#[cfg(feature = "graphical")]
pub fn monitor_config(
    event_loop_proxy: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    path: Option<PathBuf>,
) {
    use res_def::dirs::config_dir;

    if let Some(dir) = config_dir() {
        if let Some(path) = path {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = async_watch(path, event_loop_proxy, dir) {
                    error!("error: {:?}", e)
                }
            });
        }
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
            let _ = tx.send(res);
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
    dir: impl AsRef<Path> + Debug,
) -> notify::Result<()> {
    use super::file::read_config_file;

    let (mut watcher, rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    watcher.watch(dir.as_ref(), RecursiveMode::Recursive)?;

    while let Ok(res) = rx.recv() {
        match res {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|f| f.file_name() == path.as_ref().file_name())
                {
                    let (config, _) = read_config_file(Some(path.as_ref().to_path_buf()));

                    let proxy = event_loop_proxy.lock().expect("could not get proxy lock");
                    if let Err(e) =
                        proxy.send_event(KaraEvent::ReloadConfiguration(Box::new(config)))
                    {
                        error!("send event error {:?}", e);
                    } else {
                        trace!("configuration reloaded");
                    }
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
    Ok(())
}
