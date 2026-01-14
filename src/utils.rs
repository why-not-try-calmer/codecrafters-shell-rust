use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::enums::WriteFileMode;

pub fn strip_bytes(mut bytes: Vec<u8>) -> Vec<u8> {
    if matches!(bytes.first(), Some(b'\'') | Some(b'"')) {
        if let Some(pos) = bytes.iter().rposition(|&b| b == b'"' || b == b'\'') {
            bytes.remove(pos);
            bytes.remove(0);
        }
    }
    bytes
}

pub fn read_from_file<P: AsRef<Path>>(maybe_path: P) -> String {
    let mut buffer = String::new();
    let mut options = OpenOptions::new();
    let mut file = options.read(true).open(maybe_path).unwrap();
    file.read_to_string(&mut buffer).unwrap();
    buffer
}

pub fn fill_history<P: AsRef<Path>>(pathref: P, history: &mut Vec<String>) {
    let lines: Vec<String> = read_from_file(pathref)
        .lines()
        .map(|x| x.to_string())
        .collect();
    for line in lines {
        history.push(line);
    }
}

pub fn dump_history<P: AsRef<Path>>(pathref: P, history: &mut Vec<String>, mode: WriteFileMode) {
    let mut joined = history.join("\n");
    joined.push('\n');
    let contents: &[u8] = joined.as_bytes();
    match mode {
        WriteFileMode::Append => {
            write_to_file(contents, pathref, WriteFileMode::Append);
            history.clear();
        }
        WriteFileMode::OverWrite => {
            write_to_file(contents, pathref, WriteFileMode::OverWrite);
        }
    }
}

pub fn write_to_file<P: AsRef<Path>>(contents: &[u8], path: P, mode: WriteFileMode) {
    let mut options = OpenOptions::new();
    options.create(true);
    if mode == WriteFileMode::Append {
        options.append(true);
    } else {
        options.write(true).truncate(true);
    }
    let mut file = options.open(path).unwrap();
    file.write_all(contents).unwrap();
}

pub fn find_executable_on_path(cmd_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    for path in env::split_paths(&path_var) {
        let full_path = path.join(cmd_name);

        if let Ok(metadata) = full_path.metadata() {
            if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                return Some(full_path);
            }
        }
    }
    None
}

pub fn run_cmd(program: &str, args: &[&str]) -> Output {
    Command::new(program).args(args).output().unwrap()
}
