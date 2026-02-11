use colored::Colorize;

pub mod log {
    use super::*;

    pub fn info(msg: &str) {
        println!("{}", msg.dimmed());
    }

    pub fn info_important(msg: &str) {
        println!("{}", msg);
    }

    pub fn warn(msg: &str) {
        println!("{}", msg.yellow());
    }

    pub fn error(msg: &str) {
        eprintln!("{}", msg.red());
    }

    pub fn success(msg: &str) {
        println!("{}", msg.green());
    }

    /// Print a heading with green background + powerline segment (for milestones)
    pub fn heading_note(title: &str) {
        println!(
            "{}{}{}{}{}",
            "  ".on_green(),
            title.white().bold().on_green(),
            "  ".on_green(),
            "\u{E0B0}".green(),
            ""
        );
    }

    /// Print a heading with bright blue background + ⬝ prefix + powerline segment (for steps)
    pub fn heading_info(title: &str) {
        println!(
            "{}{}{}{}{}",
            "  ⬝".white().bold().on_bright_blue(),
            title.white().bold().on_bright_blue(),
            "   ".on_bright_blue(),
            "\u{E0B0}".bright_blue(),
            ""
        );
    }
}
