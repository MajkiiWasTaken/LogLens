/************************************************
* File: output.rs
* Author: Michal Švrček
*
* Colored terminal output.
*
* ver. 0.1.0
*************************************************/

use colored::Colorize;

pub fn banner() {
    println!("{}", "◆ LogLens v0.1.0".bright_blue().bold());
    println!("{}", "────────────────────────────────".dimmed());
}

pub fn heading(message: &str) {
    println!("\n{}", message.bright_cyan().bold());
}

pub fn info(message: &str) {
    println!("{} {message}", "[INFO]".blue().bold());
}

pub fn ok(message: &str) {
    println!("{} {message}", "[ OK ]".green().bold());
}

pub fn error(message: &str) {
    eprintln!("{} {message}", "[ERROR]".red().bold());
}

pub fn log_line(line: &str) {
    let lower = line.to_ascii_lowercase();

    if lower.contains("error") || lower.contains("failed") || lower.contains("fatal") {
        println!("{}", line.red());
    } else if lower.contains("warn") {
        println!("{}", line.yellow());
    } else if lower.contains("debug") || lower.contains("trace") {
        println!("{}", line.dimmed());
    } else {
        println!("{line}");
    }
}
