use std::{
    env,
    io::{self},
};

use codecrafters_shell::{commands::handle_command, enums::WriteFileMode, utils};

fn init() -> (Vec<String>, Option<String>) {
    let history_path = env::var("HISTFILE").ok();
    let mut history = Vec::new();

    history_path
        .as_ref()
        .map(|path| utils::fill_history(path, &mut history));

    (history, history_path)
}

fn main() {
    let (mut history, maybe_path) = init();

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
                    match maybe_path {
                        Some(pathref) => {
                            utils::dump_history(pathref, &mut history, WriteFileMode::OverWrite)
                        }
                        None => {}
                    }
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
