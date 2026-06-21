use owo_colors::OwoColorize;
use uv_bump::DependencyChange;

pub fn print_diff(changes: &[DependencyChange]) {
    for change in changes {
        println!("- {:<16} {}", change.name.bold(), change.old.red());
        println!("+ {:<16} {}", change.name.bold(), change.new.bright_green());
        println!();
    }

    println!(
        "{} dependency changes. Run `{}` to apply them.",
        changes.len().to_string().bold(),
        "uv-bump apply".bright_green()
    );
}
