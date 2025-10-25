use crate::config::FyrerConfig;
use colored::*;

pub async fn run_installers(config: &FyrerConfig) {
    if let Some(installers) = &config.installers {
        println!("{}", " Running installer steps...".bright_cyan().bold());
        println!(
            "{}",
            "────────────────────────────────────────────".bright_black()
        );

        for installer in installers {
            println!(
                "{} {}",
                "🔹 Running installer in:".bright_blue().bold(),
                installer.dir.bright_yellow()
            );
            println!(
                "{} {}",
                " Command:".bright_blue(),
                installer.cmd.bright_green().italic()
            );

            #[cfg(unix)]
            let mut cmd = tokio::process::Command::new("sh");
            #[cfg(unix)]
            cmd.arg("-c").arg(&installer.cmd);

            #[cfg(windows)]
            let mut cmd = tokio::process::Command::new("cmd");
            #[cfg(windows)]
            cmd.arg("/C").arg(&installer.cmd);

            cmd.current_dir(&installer.dir);
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());

            match cmd.status().await {
                Ok(status) if status.success() => {
                    println!(
                        "{} {}\n",
                        " Installer completed successfully in:"
                            .bright_green()
                            .bold(),
                        installer.dir.bright_yellow()
                    );
                }
                Ok(status) => {
                    eprintln!(
                        "{} {} {}",
                        " Installer failed in:".bright_red().bold(),
                        installer.dir.bright_yellow(),
                        format!("(Exit code: {})", status.code().unwrap_or(-1)).bright_red()
                    );
                    println!();
                }
                Err(e) => {
                    eprintln!(
                        "{} {}: {}",
                        " Failed to execute installer in:".bright_red().bold(),
                        installer.dir.bright_yellow(),
                        e.to_string().bright_red()
                    );
                    println!();
                }
            }

            println!(
                "{}",
                "────────────────────────────────────────────".bright_black()
            );
        }

        println!("{}", " All installer steps completed!".bright_cyan().bold());
        println!();
    } else {
        println!(
            "{}",
            "⚡ No installer steps defined.".bright_yellow().bold()
        );
    }
}
