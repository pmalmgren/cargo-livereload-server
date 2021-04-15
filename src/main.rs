use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use tokio::io::Result;
use tokio::sync::mpsc;

mod livereload;
mod watch;

async fn run() -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(watch::watch(tx, String::from(".")));
    tokio::spawn(livereload::server());

    while let Some(changes) = rx.recv().await {
        for change in changes.iter() {
            log::info!("Received file change: {}", change);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S.%f"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    if let Err(e) = run().await {
        log::error!("Error = {}", e);
    }
}
