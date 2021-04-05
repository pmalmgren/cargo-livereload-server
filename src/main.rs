extern crate notify;

use structopt::StructOpt;

use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use std::sync::mpsc::channel;

use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    file_path: Option<std::path::PathBuf>,
}

#[tokio::main]
pub async fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S.%f"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let args = Cli::from_args();

    let file_path = match args.file_path {
        Some(fp) => fp.to_str().unwrap().to_string(),
        None => ".".to_string(),
    };

    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).unwrap();
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
           Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
               log::info!("{:?} {:?} ({:?})", op, path, cookie)
           },
           Ok(event) => log::warn!("broken event: {:?}", event),
           Err(e) => log::error!("watch error: {:?}", e),
        }
    }
}
