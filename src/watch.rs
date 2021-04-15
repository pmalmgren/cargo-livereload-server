use tokio::io::Result;
use tokio::sync::mpsc;

use std::collections::HashSet;
use std::time::{Duration, Instant};

// for dealing with the notify library
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

fn should_send(changes: &HashSet<String>, last_change: Option<Instant>) -> bool {
    match last_change {
        None => false,
        Some(lc) => {
            let one_sec_elapsed = Instant::now().duration_since(lc) > Duration::from_secs(1);
            changes.len() > 0 && one_sec_elapsed
        }
    }
}

async fn watch_for_changes(
    rx: std::sync::mpsc::Receiver<RawEvent>,
    tx: mpsc::UnboundedSender<HashSet<String>>,
) {
    let timeout = Duration::from_millis(250);
    let mut changes: HashSet<String> = HashSet::new();
    let mut last_change: Option<Instant> = None;

    loop {
        match rx.recv_timeout(timeout) {
            Ok(RawEvent {
                path: Some(path),
                op: Ok(_op),
                cookie: _cookie,
            }) => {
                last_change = match last_change {
                    Some(lc) => Some(lc),
                    None => Some(Instant::now()),
                };
                changes.insert(path.to_str().unwrap().to_string());
            }
            Ok(event) => log::error!("broken event: {:?}", event),
            Err(e) => match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => (),
                std::sync::mpsc::RecvTimeoutError::Disconnected => {
                    log::error!("Notify disconnected");
                    break;
                }
            },
        }

        if should_send(&changes, last_change) {
            log::info!("Sending changes...");
            tx.send(changes).unwrap();
            changes = HashSet::new();
            last_change = None;
        }
    }
}

pub async fn watch(tx: mpsc::UnboundedSender<HashSet<String>>, watch_path: String) -> Result<()> {
    let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
    let mut watcher = raw_watcher(watcher_tx).expect("Error: Initializing raw watcher");
    watcher
        .watch(watch_path, RecursiveMode::Recursive)
        .expect("Error: Watcher");

    log::info!("File watcher started...");

    watch_for_changes(watcher_rx, tx).await;

    Ok(())
}
