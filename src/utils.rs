use std::env;
use std::fs::OpenOptions;
use std::io::Write;
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
