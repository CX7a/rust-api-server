use colored::*;
use std::io::{self, Write};

pub fn spinner_start(message: &str) {
    print!("{} ", message.cyan());
    io::stdout().flush().ok();
}

pub fn spinner_stop() {
    println!();
}

pub fn prompt(message: &str) -> String {
    print!("{} ", message.cyan());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn confirm(message: &str) -> bool {
    print!("{} (y/n) ", message.yellow());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes"
}

pub fn print_success(message: &str) {
    println!("{}", format!("✓ {}", message).green().bold());
}

pub fn print_error(message: &str) {
    println!("{}", format!("✗ {}", message).red().bold());
}

pub fn print_info(message: &str) {
    println!("{}", format!("ℹ {}", message).blue());
}

pub fn print_warning(message: &str) {
    println!("{}", format!("⚠ {}", message).yellow());
}
