use owo_colors::OwoColorize;
use uv_bump::DependencyChange;

pub fn print_diff(changes: &[DependencyChange]) {
    for change in changes {
        println!(
            "{} {:<16} {}{}{}",
            "-".bright_red(),
            change.name.bold(),
            change.operator.clone().unwrap_or_default().bright_red(),
            change.old.bright_red().underline(),
            change.suffix.clone().unwrap_or_default().bright_red(),
        );
        println!(
            "{} {:<16} {}{}{}",
            "+".bright_green(),
            change.name.bold(),
            change.operator.clone().unwrap_or_default().bright_green(),
            change.new.bright_green().underline(),
            change.suffix.clone().unwrap_or_default().bright_green()
        );
        println!();
    }
}
