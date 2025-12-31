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
}
