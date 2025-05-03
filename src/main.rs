use std::process::{Command, Output};
use std::io;
use colored::*;
use dialoguer::{Input, Select, Confirm};
use dialoguer::theme::ColorfulTheme;
use std::borrow::Cow;

fn get_command_string(cmd: &Command) -> String {
    let args = cmd.get_args()
        .map(|os| os.to_string_lossy())
        .collect::<Vec<Cow<str>>>();
    let program = cmd.get_program().to_string_lossy();
    let full_cmd = std::iter::once(program)
        .chain(args)
        .collect::<Vec<_>>()
        .join(" ");
    full_cmd
}

fn run_command(cmd: &mut Command) -> io::Result<Output> {
    cmd.output()
}

fn check_command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}

fn install_paru() -> anyhow::Result<()> {
    println!("{}", "Установка paru...".cyan().bold());
    let output = Command::new("sh")
        .arg("-c")
        .arg("git clone https://aur.archlinux.org/paru.git && cd paru && makepkg -si --noconfirm")
        .output()?;

    if output.status.success() {
        println!("{}", "✔ paru успешно установлен.".green().bold());
        Ok(())
    } else {
        Err(anyhow::anyhow!("Ошибка при установке paru"))
    }
}

fn execute_package_manager(action: &str, packages: &[String]) -> anyhow::Result<()> {
    for package in packages {
        let mut pacman_cmd = Command::new("sudo");
        pacman_cmd.arg("pacman");

        match action {
            "install" => {
                pacman_cmd.arg("-S").arg("--noconfirm").arg(package);
            }
            "remove" => {
                pacman_cmd.arg("-R").arg("--noconfirm").arg(package);
            }
            _ => unreachable!(),
        }

        println!("{} {}", "▶ Выполняется команда:".cyan().bold(), get_command_string(&pacman_cmd));
        let output = run_command(&mut pacman_cmd)?;

        if output.status.success() {
            continue;
        }

        if action == "install" {
            if !check_command_exists("paru") {
                return Err(anyhow::anyhow!("paru не установлен и требуется для установки {}", package));
            }

            println!("{} {} {}", "⚠ Не удалось найти пакет".yellow().bold(), package.bold(), "в репозитории pacman. Пробую через paru...".yellow().bold());

            let mut paru_cmd = Command::new("paru");
            paru_cmd.arg("-S").arg("--noconfirm").arg(package);

            println!("{} {}", "▶ Выполняется команда:".cyan().bold(), get_command_string(&paru_cmd));
            let paru_output = run_command(&mut paru_cmd)?;

            if paru_output.status.success() {
                continue;
            }

            println!(
                "{}",
                format!(
                    "❌ Ошибка при установке пакета '{}':\n{}",
                    package,
                    String::from_utf8_lossy(&paru_output.stderr)
                )
                .red()
                .bold()
            );
            return Err(anyhow::anyhow!("Ошибка установки пакета {}", package));
        } else {
            println!(
                "{}",
                format!(
                    "❌ Ошибка при удалении пакета '{}':\n{}",
                    package,
                    String::from_utf8_lossy(&output.stderr)
                )
                .red()
                .bold()
            );
            return Err(anyhow::anyhow!("Ошибка удаления пакета {}", package));
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!(
        "Welcome to {} - Simple Package Management",
        "ArtexPackageManagement".blue().bold()
    );

    if !check_command_exists("paru") {
        let install = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Пакет 'paru' не найден. Хотите установить его?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Ошибка ввода: {}", e))?;

        if !install {
            println!("{}", "Программа завершена: требуется 'paru' для дополнительной поддержки пакетов AUR.".red().bold());
            return Ok(());
        }

        install_paru()?;
    }

    let actions = vec!["Install packages", "Remove packages"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Выберите действие")
        .items(&actions)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!("Ошибка выбора: {}", e))?;

    let action = match selection {
        0 => "install",
        1 => "remove",
        _ => unreachable!(),
    };

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Введите названия пакетов через пробел")
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Ошибка ввода: {}", e))?;

    let packages: Vec<String> = input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if packages.is_empty() {
        println!("{}", "Список пакетов пуст.".yellow().bold());
        return Ok(());
    }

    execute_package_manager(action, &packages)?;

    println!("{}", "Операция успешно завершена.".green().bold());
    Ok(())
}