use crate::interpret;
use crate::parse;
use crate::utils;

pub fn do_type(args: &[&str]) {
    let supported = ["echo", "exit", "type"];
    let arg = args.get(0).unwrap();

    if supported.contains(arg) {
        eprintln!("{arg} is a shell builtin")
    } else {
        if let Some(found) = utils::find_executable_on_path(arg) {
            let full_path = found.to_str().unwrap();
            eprintln!("{arg} is {full_path}");
        } else {
            eprintln!("{arg}: not found");
        }
    }
}

pub fn do_history(args: &[&str], history: &mut Vec<String>) {
    if let Some(arg) = args.get(0) {
        if *arg != "-r" {
            eprintln!(
                "history supports only the `-r` argument, but you passed '{}' in",
                arg
            );
            return;
        }
        let maybe_path = args.get(1).unwrap();
        let lines: Vec<String> = utils::read_from_file(maybe_path)
            .lines()
            .map(|x| x.to_string())
            .collect();
        history.clear();
        for line in lines {
            history.push(line);
        }
    } else {
        for (idx, line) in history.iter().enumerate() {
            println!("{}  {}", idx + 1, line);
        }
    }
}

pub fn handle_command(cmd: &str, args: &[&str], input_str: &str, history: &mut Vec<String>) {
    match cmd.to_lowercase().as_str() {
        "type" => do_type(&args),
        "history" => do_history(&args, history),
        _ => {
            let commands = parse::run_parser(input_str);
            interpret::interpret_command(commands);
        }
    }
}
