use crate::config::Service;
use colored::Colorize;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Child;
use tokio::{
    io::AsyncBufReadExt,
    process::Command,
    sync::mpsc,
    time::{Duration, sleep},
};

pub async fn runner(service: Service, color: colored::Color) {
    if service.watch.unwrap_or(false) {
        run_with_watch(service, color).await;
    } else {
        run_once(service, color).await;
    }
}

async fn run_with_watch(service: Service, color: colored::Color) {
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

    let mut child: Option<Child> = start_service(&service, color).await;

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
                    kill_process(&mut c, &service.name, color).await;
                    sleep(Duration::from_millis(500)).await;
                }
                child = start_service(&service, color).await;

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
                child = start_service(&service, color).await;
            }
        }
    }
}

async fn start_service(service: &Service, color: colored::Color) -> Option<Child> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&service.cmd);
    cmd.current_dir(&service.dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(envs) = &service.env {
        for (key, value) in envs {
            cmd.env(key, value);
        }
    }

    println!(
        "{} {}",
        format!("[{}] Starting service...", service.name)
            .color(color)
            .bold(),
        service.cmd
    );

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "{} {}",
                format!("[{}] Failed to start service:", service.name)
                    .color(color)
                    .bold(),
                e
            );
            return None;
        }
    };

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let name = service.name.clone();
    let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
    let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

    let name_prefix = format!("[{}] ", name).color(color).bold();
    let out_prefix = name_prefix.clone();
    let err_prefix = name_prefix.clone();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("{} {}", out_prefix, line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("{} {}", err_prefix, line.red());
        }
    });

    Some(child)
}

async fn run_once(service: Service, color: colored::Color) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&service.cmd);
    cmd.current_dir(&service.dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(envs) = &service.env {
        for (key, value) in envs {
            cmd.env(key, value);
        }
    }

    println!(
        "{} {}",
        format!("[{}] Starting service...", service.name)
            .color(color)
            .bold(),
        service.cmd
    );

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "{} {}",
                format!("[{}] Failed to start service:", service.name)
                    .color(color)
                    .bold(),
                e
            );
            return;
        }
    };
    let stdout = child.stdout.expect("Failed to capture stdout");
    let stderr = child.stderr.expect("Failed to capture stderr");

    let name = service.name.clone();

    let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
    let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

    let name_prefix = format!("[{}] ", name).color(color).bold();
    let out_prefix = name_prefix.clone();
    let err_prefix = name_prefix.clone();

    let out_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("{} {}", out_prefix, line);
        }
    });

    let err_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("{} {}", err_prefix, line.red());
        }
    });

    let _ = tokio::join!(out_task, err_task);
}

async fn kill_process(child: &mut Child, service_name: &str, color: colored::Color) {
    if let Some(pid) = child.id() {
        let output = Command::new("ps")
            .arg("-o")
            .arg("pid=")
            .arg("--ppid")
            .arg(pid.to_string())
            .output()
            .await;

        match output {
            Ok(output) => {
                let child_pids = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter_map(|l| l.trim().parse::<u32>().ok())
                    .collect::<Vec<_>>();

                for child_pid in &child_pids {
                    let _ = Command::new("kill")
                        .arg("-9")
                        .arg(child_pid.to_string())
                        .status()
                        .await;
                }

                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .status()
                    .await;
            }
            Err(e) => {
                eprintln!(
                    "{} {}",
                    format!("[{}] Failed to fetch child processes:", service_name)
                        .color(color)
                        .bold(),
                    e
                );

                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .status()
                    .await;
            }
        }
    }
}
