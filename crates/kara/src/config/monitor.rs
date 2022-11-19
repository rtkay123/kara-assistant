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

use super::{read_config_file, read_file, Configuration};

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
                            current_path.push("kara/kara.toml");

                            parse_file(&path, Arc::clone(&event_loop_proxy), "kara/kara.toml");
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

pub fn parse_file(
    base_path: impl AsRef<Path>,
    event_loop_proxy: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    path: impl AsRef<Path>,
) {
    let send_config_file = |event_loop_proxy: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
                            configuration: Configuration| {
        let proxy = event_loop_proxy.lock().expect("could not get proxy lock");
        if let Err(e) = proxy.send_event(KaraEvent::ReloadConfiguration(Box::new(configuration))) {
            error!("send event error {:?}", e);
        } else {
            trace!("configuration reloaded");
        }
    };

    let mut current_path = base_path.as_ref().to_path_buf();
    current_path.push(&path);

    match read_file(&current_path) {
        Ok(Ok(config)) => {
            send_config_file(event_loop_proxy, config);
        }
        Ok(Err(e)) => {
            error!(
                path = current_path.display().to_string(),
                "{e}, trying fallback"
            );
            parse_file(base_path, event_loop_proxy, "kara.toml");
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                let config = read_config_file();
                send_config_file(event_loop_proxy, config);
            } else {
                error!(
                    path = base_path.as_ref().display().to_string(),
                    "{e}, using default"
                );
            }
        }
    }
}
