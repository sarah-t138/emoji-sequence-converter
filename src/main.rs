mod convert;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

#[derive(Debug, PartialEq)]
enum Direction {
    ToGlyphs,
    ToCodepoints,
}

#[derive(Debug, PartialEq)]
enum ArgsOutcome {
    Help,
    Run {
        direction: Direction,
        path: Option<String>,
    },
    MissingDirection,
    UnexpectedArgument(String),
}

// Pulled out of main so the parsing rules (last --to-* wins, first bare
// argument is the path, anything after that is an error) can be tested
// without going through env::args() or process exit codes.
fn parse_args(args: &[String]) -> ArgsOutcome {
    let mut direction: Option<Direction> = None;
    let mut path: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--to-glyphs" => direction = Some(Direction::ToGlyphs),
            "--to-codepoints" => direction = Some(Direction::ToCodepoints),
            "-h" | "--help" => return ArgsOutcome::Help,
            other if path.is_none() => path = Some(other.to_string()),
            other => return ArgsOutcome::UnexpectedArgument(other.to_string()),
        }
    }

    match direction {
        Some(direction) => ArgsOutcome::Run { direction, path },
        None => ArgsOutcome::MissingDirection,
    }
}

fn print_usage(program: &str) {
    eprintln!("usage: {program} <--to-glyphs|--to-codepoints> [FILE]");
    eprintln!();
    eprintln!("Converts emoji sequences between hex codepoint notation");
    eprintln!("(e.g. \"1F468 200D 1F469\") and literal emoji characters,");
    eprintln!("one sequence per line. Reads FILE if given, otherwise stdin.");
}

fn read_input(path: Option<&str>) -> io::Result<String> {
    match path {
        Some(p) => fs::read_to_string(p),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let program = args
        .first()
        .map(String::as_str)
        .unwrap_or("emojiconv")
        .to_string();

    let (direction, path) = match parse_args(&args[1..]) {
        ArgsOutcome::Help => {
            print_usage(&program);
            return ExitCode::SUCCESS;
        }
        ArgsOutcome::Run { direction, path } => (direction, path),
        ArgsOutcome::MissingDirection => {
            print_usage(&program);
            return ExitCode::FAILURE;
        }
        ArgsOutcome::UnexpectedArgument(arg) => {
            eprintln!("{program}: unexpected argument '{arg}'");
            print_usage(&program);
            return ExitCode::FAILURE;
        }
    };

    let input = match read_input(path.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{program}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut had_error = false;

    for (i, line) in input.lines().enumerate() {
        let result: Result<String, String> = match direction {
            Direction::ToGlyphs => convert::to_glyphs(line),
            Direction::ToCodepoints => Ok(convert::to_codepoints(line)),
        };

        match result {
            Ok(converted) => {
                if writeln!(out, "{converted}").is_err() {
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                eprintln!("{program}: line {}: {e}", i + 1);
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_args_to_glyphs_no_path() {
        assert_eq!(
            parse_args(&args(&["--to-glyphs"])),
            ArgsOutcome::Run {
                direction: Direction::ToGlyphs,
                path: None,
            }
        );
    }

    #[test]
    fn parse_args_to_codepoints_with_path() {
        assert_eq!(
            parse_args(&args(&["--to-codepoints", "sequences.txt"])),
            ArgsOutcome::Run {
                direction: Direction::ToCodepoints,
                path: Some("sequences.txt".to_string()),
            }
        );
    }

    #[test]
    fn parse_args_path_before_flag_still_works() {
        assert_eq!(
            parse_args(&args(&["sequences.txt", "--to-glyphs"])),
            ArgsOutcome::Run {
                direction: Direction::ToGlyphs,
                path: Some("sequences.txt".to_string()),
            }
        );
    }

    #[test]
    fn parse_args_last_direction_flag_wins() {
        assert_eq!(
            parse_args(&args(&["--to-glyphs", "--to-codepoints"])),
            ArgsOutcome::Run {
                direction: Direction::ToCodepoints,
                path: None,
            }
        );
    }

    #[test]
    fn parse_args_help_short_and_long() {
        assert_eq!(parse_args(&args(&["-h"])), ArgsOutcome::Help);
        assert_eq!(parse_args(&args(&["--help"])), ArgsOutcome::Help);
    }

    #[test]
    fn parse_args_help_wins_even_after_other_flags() {
        assert_eq!(
            parse_args(&args(&["--to-glyphs", "--help"])),
            ArgsOutcome::Help
        );
    }

    #[test]
    fn parse_args_no_direction_is_missing() {
        assert_eq!(parse_args(&args(&[])), ArgsOutcome::MissingDirection);
        assert_eq!(
            parse_args(&args(&["sequences.txt"])),
            ArgsOutcome::MissingDirection
        );
    }

    #[test]
    fn parse_args_second_bare_argument_is_unexpected() {
        assert_eq!(
            parse_args(&args(&["--to-glyphs", "a.txt", "b.txt"])),
            ArgsOutcome::UnexpectedArgument("b.txt".to_string())
        );
    }
}
