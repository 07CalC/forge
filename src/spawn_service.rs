use crate::config::Service;
use colored::Colorize;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

pub async fn spawn_service(service: &Service, color: colored::Color, wait: bool) -> Option<Child> {
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

    let mut child = cmd.spawn().ok()?;

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

    if wait {
        let _ = child.wait().await;
        return None;
    }

    Some(child)
}
