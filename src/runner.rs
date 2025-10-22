use crate::config::Service;
use colored::Colorize;
use std::process::Stdio;
use tokio::{io::AsyncBufReadExt, process::Command};

pub async fn runner(service: Service, color: colored::Color) {
    let child = Command::new("sh")
        .arg("-c")
        .arg(&service.cmd)
        .current_dir(&service.dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(format!("Failed to start service: {}", &service.name).as_str());
    println!(
        "{} {}",
        format!("[{}] Starting service...", service.name)
            .color(color)
            .bold(),
        service.cmd
    );
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
