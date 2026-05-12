use console::style;

pub mod log {
    use super::*;

    pub fn info(msg: &str) {
        println!("{}", style(msg).dim());
    }

    pub fn info_important(msg: &str) {
        println!("{}", msg);
    }

    pub fn warn(msg: &str) {
        println!("{}", style(msg).yellow());
    }

    pub fn error(msg: &str) {
        eprintln!("{}", style(msg).red());
    }

    pub fn success(msg: &str) {
        println!("{}", style(msg).green());
    }
}
