use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
#[allow(unused_imports)]
use std::io::{self, Write};

pub fn ts() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

pub fn log_info(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(Color::Blue));
    print!("[lisa {}] ", ts());
    let _ = crossterm::execute!(stdout, ResetColor);
    println!("{}", msg);
}

pub fn log_success(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(Color::Green));
    print!("[lisa {}] ", ts());
    let _ = crossterm::execute!(stdout, ResetColor);
    println!("{}", msg);
}

pub fn log_warn(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(Color::Yellow));
    print!("[lisa {}] ", ts());
    let _ = crossterm::execute!(stdout, ResetColor);
    println!("{}", msg);
}

pub fn log_error(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(Color::Red));
    print!("[lisa {}] ", ts());
    let _ = crossterm::execute!(stdout, ResetColor);
    println!("{}", msg);
}

pub fn log_phase(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(Color::Cyan));
    print!("[lisa {}] ", ts());
    let _ = crossterm::execute!(stdout, ResetColor);
    println!("━━━ {} ━━━", msg);
}

pub fn print_bold(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Bold));
    print!("{}", msg);
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Reset));
}

pub fn println_bold(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Bold));
    println!("{}", msg);
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Reset));
}

pub fn print_colored(msg: &str, color: Color) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(color));
    print!("{}", msg);
    let _ = crossterm::execute!(stdout, ResetColor);
}

pub fn println_colored(msg: &str, color: Color) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetForegroundColor(color));
    println!("{}", msg);
    let _ = crossterm::execute!(stdout, ResetColor);
}

pub fn print_separator() {
    println_bold("═══════════════════════════════════════════════════════");
}

pub fn print_dim(msg: &str) {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Dim));
    print!("{}", msg);
    let _ = crossterm::execute!(stdout, SetAttribute(Attribute::Reset));
}
