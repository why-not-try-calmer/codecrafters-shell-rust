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

pub fn handle_command(cmd: &str, args: &[&str], input_str: &str) {
    match cmd.to_lowercase().as_str() {
        "type" => do_type(&args),
        _ => {
            let commands = parse::run_parser(input_str);
            interpret::interpret_command(commands);
        }
    }
}
