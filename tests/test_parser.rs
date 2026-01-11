#[cfg(test)]
mod test_parser {
    use codecrafters_shell::enums::{Command, RedirectMode, WriteFileMode};
    use codecrafters_shell::parse::run_parser;

    #[test]
    fn test_parser_1() {
        let input_string = String::from("echo '123' 2>> file.txt");
        println!("Parsing {input_string}");
        let results = run_parser(&input_string);
        println!("Got: {:#?}", results);
        assert_eq!(
            results,
            vec![
                Command::Program {
                    cmd: String::from("echo"),
                    args: vec![String::from("123")]
                },
                Command::RedirectTo {
                    path: String::from("file.txt"),
                    redirect_mode: RedirectMode::StdErr,
                    write_mode: WriteFileMode::Append
                }
            ]
        )
    }

    #[test]
    fn test_parser_2() {
        let input_string = String::from("cat file.txt | head 1 1> file2.txt");
        println!("Parsing {input_string}");
        let results = run_parser(&input_string);
        println!("Got: {:#?}", results);
        assert_eq!(
            results,
            vec![
                Command::Pipe {
                    programs: vec![
                        Box::new(Command::Program {
                            cmd: String::from("cat"),
                            args: vec![String::from("file.txt")]
                        }),
                        Box::new(Command::Program {
                            cmd: String::from("head"),
                            args: vec![String::from("1")]
                        })
                    ]
                },
                Command::RedirectTo {
                    path: String::from("file2.txt"),
                    redirect_mode: RedirectMode::StdOut,
                    write_mode: WriteFileMode::OverWrite
                }
            ]
        )
    }

    #[test]
    fn test_parser_3() {
        let input_string = String::from("tail -f tests/testdata.txt | head -n 5");
        println!("Parsing {input_string}");
        let results = run_parser(&input_string);
        println!("Got: {:#?}", results);
        assert_eq!(
            results,
            vec![Command::Pipe {
                programs: vec![
                    Box::new(Command::Program {
                        cmd: String::from("tail"),
                        args: vec![String::from("-f"), String::from("tests/testdata.txt")]
                    }),
                    Box::new(Command::Program {
                        cmd: String::from("head"),
                        args: vec![String::from("-n"), String::from("5")]
                    })
                ]
            }]
        )
    }

    #[test]
    fn test_parser_4() {
        let input_string = String::from("ls -1 nonexistent 2>> /tmp/dog/cow.md");
        println!("Parsing {input_string}");
        let results = run_parser(&input_string);
        println!("Got: {:#?}", results);
        assert_eq!(
            results,
            vec![
                Command::Program {
                    cmd: String::from("ls"),
                    args: vec![String::from("-1"), String::from("nonexistent")]
                },
                Command::RedirectTo {
                    path: String::from("/tmp/dog/cow.md"),
                    redirect_mode: RedirectMode::StdErr,
                    write_mode: WriteFileMode::Append
                }
            ]
        )
    }

    #[test]
    fn test_parser_5() {
        let input_string = String::from("ls | type exit");
        println!("Parsing {input_string}");
        let results = run_parser(&input_string);
        println!("Got: {:#?}", results);
        assert_eq!(
            results,
            vec![Command::Pipe {
                programs: vec![
                    Box::new(Command::Program {
                        cmd: String::from("ls"),
                        args: vec![]
                    }),
                    Box::new(Command::Program {
                        cmd: String::from("type"),
                        args: vec![String::from("exit")]
                    })
                ]
            }]
        )
    }
}
