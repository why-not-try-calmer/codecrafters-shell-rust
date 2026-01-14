use std::io::{self};

use codecrafters_shell::commands::handle_command;

fn main() {
    let mut history: Vec<String> = vec![];
    loop {
        eprint!("$ ");
        let mut input_str = String::new();
        match io::stdin().read_line(&mut input_str) {
            Ok(0) => {
                eprintln!();
                break;
            }
            Ok(_) => {
                history.push(input_str.clone().trim().to_string());
                let cmd_args: Vec<&str> = input_str.split_whitespace().collect();
                let (cmd, args) = cmd_args.split_first().unwrap();
                if cmd.eq_ignore_ascii_case("exit") {
                    break;
                }
                handle_command(cmd, args, &input_str, &mut history);
            }
            Err(error) => {
                eprintln!("Error reading input: {error}");
                break;
            }
        }
    }
}
