#[derive(Debug, Clone, PartialEq)]
pub enum WriteFileMode {
    OverWrite,
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectMode {
    StdOut,
    StdErr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Program {
        cmd: String,
        args: Vec<String>,
    },
    Pipe {
        programs: Vec<Box<Command>>,
    },
    RedirectTo {
        path: String,
        redirect_mode: RedirectMode,
        write_mode: WriteFileMode,
    },
}
