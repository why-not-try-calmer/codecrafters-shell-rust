use crate::enums::{Command, RedirectMode, WriteFileMode};

pub fn run_parser(input: &str) -> Vec<Command> {
    let mut parser = Parser::new();

    for c in input.chars() {
        parser.handle_char(c);
    }

    parser.finish()
}

struct Parser {
    commands: Vec<Command>,
    current_cmd: Option<Command>,
    last_char: char,
    pending_fd_char: Option<char>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            commands: vec![],
            current_cmd: None,
            last_char: ' ',
            pending_fd_char: None,
        }
    }

    fn handle_char(&mut self, c: char) {
        match c {
            '\'' | '"' | '\n' => {}
            '>' => self.handle_redirect(),
            '1' | '2' => self.handle_fd_marker(c),
            '|' => self.handle_pipe(),
            ';' => self.handle_semicolon(),
            ' ' => self.handle_space(),
            _ => self.handle_regular_char(c),
        }

        self.last_char = c;
    }

    fn handle_redirect(&mut self) {
        if let Some(fd_char) = self.pending_fd_char.take() {
            self.finalize_current_command();
            self.current_cmd = Some(self.create_redirect_command(fd_char));
        } else {
            if self.try_convert_to_append() {
                return;
            }

            self.finalize_current_command();
            self.current_cmd = Some(self.create_redirect_command('1'));
        }
    }

    fn try_convert_to_append(&mut self) -> bool {
        if let Some(Command::RedirectTo { write_mode, .. }) = &mut self.current_cmd {
            if *write_mode == WriteFileMode::OverWrite && self.last_char == '>' {
                *write_mode = WriteFileMode::Append;
                return true;
            }
        }
        false
    }

    fn create_redirect_command(&self, fd_char: char) -> Command {
        Command::RedirectTo {
            path: String::new(),
            redirect_mode: if fd_char == '1' {
                RedirectMode::StdOut
            } else {
                RedirectMode::StdErr
            },
            write_mode: WriteFileMode::OverWrite,
        }
    }

    fn handle_fd_marker(&mut self, c: char) {
        self.flush_pending_fd_char();
        self.pending_fd_char = Some(c);
    }

    fn handle_pipe(&mut self) {
        self.flush_pending_fd_char();

        if let Some(current) = self.current_cmd.take() {
            self.current_cmd = match current {
                // If already a Pipe, add to it
                Command::Pipe { mut programs } => {
                    programs.push(Box::new(Command::Program {
                        cmd: String::new(),
                        args: vec![],
                    }));
                    Some(Command::Pipe { programs })
                }
                // Otherwise, create a new Pipe with 2 programs
                _ => Some(Command::Pipe {
                    programs: vec![
                        Box::new(current),
                        Box::new(Command::Program {
                            cmd: String::new(),
                            args: vec![],
                        }),
                    ],
                }),
            };
        }
    }

    fn handle_semicolon(&mut self) {
        self.flush_pending_fd_char();
        self.finalize_current_command();
        self.current_cmd = Some(Command::Program {
            cmd: String::new(),
            args: vec![],
        });
    }

    fn handle_space(&mut self) {
        self.flush_pending_fd_char();

        if self.should_append_space() {
            if let Some(cmd) = &mut self.current_cmd {
                cmd.append_char(' ');
            }
        }
    }

    fn should_append_space(&self) -> bool {
        match &self.current_cmd {
            Some(Command::Program { .. }) => self
                .current_cmd
                .as_ref()
                .map_or(false, |cmd| !cmd.is_empty()),
            Some(Command::Pipe { .. }) => {
                self.current_cmd
                    .as_ref()
                    .map_or(false, |cmd| !cmd.is_empty())
                    && self.last_char != ' '
            }
            _ => false,
        }
    }

    fn handle_regular_char(&mut self, c: char) {
        self.flush_pending_fd_char();

        if let Some(cmd) = &mut self.current_cmd {
            cmd.append_char(c);
        } else {
            self.current_cmd = Some(Command::Program {
                cmd: c.to_string(),
                args: vec![],
            });
        }
    }

    fn finalize_current_command(&mut self) {
        if let Some(mut cmd) = self.current_cmd.take() {
            cmd.trim_remove_empty_args();
            self.commands.push(cmd);
        }
    }

    fn flush_pending_fd_char(&mut self) {
        if let Some(fd_char) = self.pending_fd_char.take() {
            if let Some(cmd) = &mut self.current_cmd {
                cmd.append_char(fd_char);
            }
        }
    }

    fn finish(mut self) -> Vec<Command> {
        self.flush_pending_fd_char();

        if let Some(cmd) = self.current_cmd {
            self.commands.push(cmd);
        }

        for cmd in self.commands.iter_mut() {
            cmd.trim_path();
            cmd.trim_remove_empty_args();
        }

        self.commands
    }
}

impl Command {
    fn append_char(&mut self, c: char) {
        match self {
            Command::RedirectTo { path, .. } => {
                path.push(c);
            }
            Command::Program { cmd, args } => {
                append_to_program(c, cmd, args);
            }
            Command::Pipe { programs } => {
                // Append to the last program in the pipe
                if let Some(last_program) = programs.last_mut() {
                    last_program.append_char(c);
                }
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Command::Program { cmd, args } => cmd.is_empty() && args.is_empty(),
            Command::Pipe { programs } => programs.iter().all(|p| p.is_empty()),
            Command::RedirectTo { path, .. } => path.is_empty(),
        }
    }

    fn trim_remove_empty_args(&mut self) {
        match self {
            Command::Program { cmd, args } => {
                *cmd = cmd.trim().to_string();
                args.iter_mut()
                    .for_each(|arg| *arg = arg.trim().to_string());
                args.retain(|arg| !arg.is_empty());
            }
            Command::Pipe { programs } => {
                for program in programs.iter_mut() {
                    program.trim_remove_empty_args();
                }
            }
            _ => {}
        }
    }

    fn trim_path(&mut self) {
        if let Command::RedirectTo { path, .. } = self {
            *path = path.trim().to_string();
        }
    }

    pub fn get_cmd_args(&self) -> (String, Vec<String>) {
        match self {
            Command::Program { cmd, args } => (cmd.to_string(), args.to_vec()),
            _ => panic!("get_cmd_args only supported for Program commands"),
        }
    }
}

fn append_to_program(c: char, cmd: &mut String, args: &mut Vec<String>) {
    if cmd.is_empty() {
        cmd.push(c);
    } else if c == ' ' {
        if !args.is_empty() || !cmd.is_empty() {
            args.push(String::new());
        }
    } else if let Some(last_arg) = args.last_mut() {
        last_arg.push(c);
    } else {
        cmd.push(c);
    }
}
