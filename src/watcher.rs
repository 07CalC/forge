use crate::config::Service;
use crate::kill_process::kill_process;
use crate::spawn_service::spawn_service;
use colored::Colorize;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::process::Child;
use tokio::{
    sync::mpsc,
    time::{Duration, sleep},
};

pub async fn run_with_watch(service: Service, color: colored::Color, max_name_len: usize) {
    let (tx, mut rx) = mpsc::channel(1);
    let watch_dir = Path::new(&service.dir);

    let tx_clone = tx.clone();
    let _watcher = {
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if res.is_ok() {
                    let _ = tx_clone.try_send(());
                }
            },
            Config::default(),
        )
        .unwrap_or_else(|e| {
            eprintln!("Failed to create file watcher: {}", e);
            std::process::exit(1);
        });

        watcher
            .watch(watch_dir, RecursiveMode::Recursive)
            .unwrap_or_else(|e| {
                eprintln!("Failed to watch directory {}: {}", service.dir, e);
                std::process::exit(1);
            });
        watcher
    };

    let out_prefix = format!("[{}]", service.name);
    let padded_name = format!("{:<width$}", out_prefix.clone(), width = max_name_len);

    println!(
        "├─{} ➤ Watching {} for changes...",
        padded_name.color(color).bold(),
        service.dir
    );

    let mut child: Option<Child> = spawn_service(&service, color, false, max_name_len).await;

    loop {
        tokio::select! {
            _ = rx.recv() => {
                if let Some(mut c) = child.take() {
                    let padded_name = format!("{:<width$}", out_prefix.clone(), width = max_name_len);
                    println!(
                        "├─{} ➤ File changed, restarting service...",
                        padded_name.color(color).bold()
                    );
                    kill_process(&mut c).await;
                    sleep(Duration::from_millis(500)).await;
                }
                child = spawn_service(&service, color, false, max_name_len).await;

                sleep(Duration::from_millis(500)).await;
                while rx.try_recv().is_ok() {}
            }
            _ = async {
                if let Some(ref mut c) = child {
                    c.wait().await
                } else {
                    std::future::pending().await
                }
            } => {
                let padded_name = format!("{:<width$}", out_prefix.clone(), width = max_name_len);
                println!(
                    "├─{} ➤ Service exited, restarting...",
                    padded_name.color(color).bold()
                );
                sleep(Duration::from_millis(1000)).await;
                child = spawn_service(&service, color, false, max_name_len).await;
            }
        }
    }
}
