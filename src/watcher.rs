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

pub async fn run_with_watch(service: Service, color: colored::Color) {
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

    println!(
        "{}",
        format!("[{}] Watching {} for changes...", service.name, service.dir)
            .color(color)
            .bold()
    );

    let mut child: Option<Child> = spawn_service(&service, color, false).await;

    loop {
        tokio::select! {
            _ = rx.recv() => {
                if let Some(mut c) = child.take() {
                    println!(
                        "{}",
                        format!("[{}] File changed, restarting service...", service.name)
                            .color(color)
                            .bold()
                    );
                    kill_process(&mut c).await;
                    sleep(Duration::from_millis(500)).await;
                }
                child = spawn_service(&service, color, false).await;

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
                println!(
                    "{}",
                    format!("[{}] Service exited, restarting...", service.name)
                        .color(color)
                        .bold()
                );
                sleep(Duration::from_millis(1000)).await;
                child = spawn_service(&service, color, false).await;
            }
        }
    }
}
