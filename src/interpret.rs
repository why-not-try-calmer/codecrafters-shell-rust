use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::process::{Command as ProcessCommand, Output, Stdio};
use std::thread;

use crate::enums::{Command, RedirectMode, WriteFileMode};
use crate::utils;

pub fn interpret_command(commands: Vec<Command>) {
    let mut last_output: Option<Output> = None;

    for command in commands {
        match command {
            Command::Program { cmd, args } => {
                last_output = execute_program(&cmd, args);
            }
            Command::Pipe { programs } => {
                last_output = execute_pipeline(programs);
            }
            Command::RedirectTo {
                path,
                redirect_mode,
                write_mode,
            } => {
                if let Some(output) = last_output.take() {
                    handle_redirect(output, path, redirect_mode, write_mode);
                }
            }
        }
    }

    if let Some(output) = last_output {
        write_output(&output.stdout);
    }
}

fn execute_program(cmd: &str, args: Vec<String>) -> Option<Output> {
    match utils::find_executable_on_path(cmd) {
        Some(program_path) => {
            let program_executable = program_path.file_name().unwrap().to_str().unwrap();
            let program_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            Some(utils::run_cmd(program_executable, &program_args))
        }
        None => {
            eprintln!("{}: command not found", cmd);
            None
        }
    }
}

fn execute_pipeline(programs: Vec<Box<Command>>) -> Option<Output> {
    if programs.is_empty() {
        return None;
    }

    // Validate all commands first
    let executables: Vec<(String, Vec<String>)> = programs
        .iter()
        .filter_map(|program| {
            let (cmd, args) = program.get_cmd_args();

            // Check for builtins
            if cmd == "type" {
                let output = args.join("");
                eprintln!("{} is a shell builtin", output);
                return None;
            }

            // Find executable
            match utils::find_executable_on_path(&cmd) {
                Some(path) => {
                    let executable = path.file_name().unwrap().to_str().unwrap().to_string();
                    Some((executable, args))
                }
                None => {
                    eprintln!("{}: not found", cmd);
                    None
                }
            }
        })
        .collect();

    if executables.len() != programs.len() {
        // One or more commands failed to resolve
        return None;
    }

    // Execute the pipeline
    if executables.len() == 1 {
        // Single command, just execute it
        let (exe, args) = &executables[0];
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        return Some(utils::run_cmd(exe, &args_refs));
    }

    // Spawn all processes in the pipeline
    let mut children = Vec::new();
    let mut previous_stdout: Option<std::process::ChildStdout> = None;

    for (i, (exe, args)) in executables.iter().enumerate() {
        let is_last = i == executables.len() - 1;
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let stdin = if let Some(prev_out) = previous_stdout.take() {
            Stdio::from(prev_out)
        } else {
            Stdio::inherit()
        };

        let mut child = ProcessCommand::new(exe)
            .args(&args_refs)
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        // Take stdout for next process BEFORE pushing to children
        if !is_last {
            previous_stdout = child.stdout.take();
        }

        children.push(child);
    }

    // NOW all processes are spawned and connected via pipes
    // Get the last process and start reading its output
    let mut last_child = children.pop().unwrap();

    // DON'T take stdout/stderr until we're ready to read
    // The pipe connections are already established in the spawn
    let stdout = last_child.stdout.take().unwrap();
    let stderr = last_child.stderr.take().unwrap();

    // Stream stdout in real-time in a separate thread
    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut stdout_data = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    println!("{}", line);
                    stdout_data.extend_from_slice(line.as_bytes());
                    stdout_data.push(b'\n');
                }
                Err(_) => break,
            }
        }
        stdout_data
    });

    // Stream stderr in real-time in a separate thread
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut stderr_data = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                // eprintln!("{}", line);
                stderr_data.extend_from_slice(line.as_bytes());
                stderr_data.push(b'\n');
            }
        }
        stderr_data
    });

    // Collect output from threads (these will block until pipes close)
    let stdout_data = stdout_handle.join().unwrap();
    let stderr_data = stderr_handle.join().unwrap();

    // NOW wait for last child
    let status = last_child.wait().unwrap();

    // Wait for remaining children
    for mut child in children.into_iter().rev() {
        let _ = child.wait();
    }

    Some(Output {
        status,
        stdout: stdout_data,
        stderr: stderr_data,
    })
}

fn handle_redirect(
    output: Output,
    path: String,
    redirect_mode: RedirectMode,
    write_mode: WriteFileMode,
) {
    let clean_stdout = utils::strip_bytes(output.stdout);
    let clean_stderr = utils::strip_bytes(output.stderr);

    let to_file = match redirect_mode {
        RedirectMode::StdErr => {
            io::stdout().write_all(clean_stdout.as_slice()).unwrap();
            clean_stderr
        }
        RedirectMode::StdOut => {
            io::stderr().write_all(clean_stderr.as_slice()).unwrap();
            clean_stdout
        }
    };

    utils::write_to_file(&to_file, path, write_mode);
}

fn write_output(stdout: &[u8]) {
    let clean_out = utils::strip_bytes(stdout.to_vec());
    io::stdout().write_all(clean_out.as_slice()).unwrap();
}
