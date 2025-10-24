use crate::{colors::COLORS, parser::load_config};

mod colors;
mod config;
mod kill_process;
mod parser;
mod print_banner;
mod runner;
mod spawn_service;
mod watcher;

#[tokio::main]
async fn main() {
    clearscreen::clear().expect("Failed to clear the screen");
    let config = load_config("fyrer.yml");
    let mut handles = vec![];
    print_banner::print_banner();
    for (i, service) in config.services.into_iter().enumerate() {
        let color = COLORS[i % COLORS.len()];
        let handle = tokio::spawn(runner::runner(service, color));
        handles.push(handle);
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    println!("\nShutting down services...");
}
