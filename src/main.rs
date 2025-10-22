use crate::parser::load_config;

mod config;
mod parser;
mod runner;

#[tokio::main]
async fn main() {
    let config = load_config("forge.yml");
    let colors = vec![
        colored::Color::Green,
        colored::Color::Yellow,
        colored::Color::Blue,
        colored::Color::Magenta,
        colored::Color::Cyan,
        colored::Color::White,
    ];
    let mut handles = vec![];

    for (i, service) in config.services.into_iter().enumerate() {
        let color = colors[i % colors.len()];
        let handle = tokio::spawn(runner::runner(service, color));
        handles.push(handle);
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    println!("\nShutting down services...");
}
