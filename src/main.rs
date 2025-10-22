use crate::parser::load_config;

mod config;
mod parser;
mod print_banner;
mod runner;

#[tokio::main]
async fn main() {
    clearscreen::clear().expect("Failed to clear the screen");
    let config = load_config("fyrer.yml");
    let colors = vec![
        colored::Color::Green,
        colored::Color::Yellow,
        colored::Color::Blue,
        colored::Color::Magenta,
        colored::Color::Cyan,
        colored::Color::White,
        colored::Color::BrightMagenta,
    ];
    let mut handles = vec![];
    print_banner::print_banner();
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
