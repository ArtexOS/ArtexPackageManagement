use std::process::Command;
use colored::*;
use dialoguer::{Input, Select};
use dialoguer::theme::ColorfulTheme;

fn execute_pacman_command(action: &str, packages: &[String]) {
    let mut command = Command::new("sudo");
    command.arg("pacman");

    match action {
        "install" => {
            command.arg("-S").arg("--noconfirm");
        }
        "remove" => {
            command.arg("-R").arg("--noconfirm");
        }
        _ => unreachable!(),
    }

    for package in packages {
        command.arg(package);
    }

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                println!(
                    "{}",
                    format!(
                        "{} {} завершено успешно.",
                        "✔".green().bold(),
                        action.green().bold()
                    ));
            } else {
                println!(
                    "{}",
                    format!(
                        "Ошибка при {} пакетов:\n{}",
                        action,
                        String::from_utf8_lossy(&output.stderr)
                    )
                        .red()
                        .bold()
                );
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("Ошибка выполнения команды: {}", e).red().bold()
            );
        }
    }
}

fn main() {
    println!(
        "Welcome to {} - Simple Package Management",
        "ArtexPackageManagement".blue().bold()
    );

    let actions = vec!["Install packages", "Remove packages"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Выберите действие")
        .items(&actions)
        .default(0)
        .interact()
        .unwrap();

    let action = match selection {
        0 => "install",
        1 => "remove",
        _ => unreachable!(),
    };

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Введите названия пакетов через пробел")
        .interact_text()
        .unwrap();

    let packages: Vec<String> = input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if packages.is_empty() {
        println!("{}", "Список пакетов пуст.".yellow().bold());
        return;
    }

    execute_pacman_command(action, &packages);
}
