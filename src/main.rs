#![warn(missing_docs)]

//! crow (command row) is a CLI tool to help you memorize CLI commands by saving them with a unique description.
//! Whenever you can't remember a certain command you can then use crow to fuzzy search commands by their description.
//! (NOTE: this tool currently only works on UNIX systems!)

use std::process;

fn main() {
    if let Err(e) = crow::run() {
        println!("Application error: {}", e);

        process::exit(1);
    };
}
