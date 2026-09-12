mod convert;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

enum Direction {
    ToGlyphs,
    ToCodepoints,
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

    let mut direction: Option<Direction> = None;
    let mut path: Option<String> = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--to-glyphs" => direction = Some(Direction::ToGlyphs),
            "--to-codepoints" => direction = Some(Direction::ToCodepoints),
            "-h" | "--help" => {
                print_usage(&program);
                return ExitCode::SUCCESS;
            }
            other if path.is_none() => path = Some(other.to_string()),
            other => {
                eprintln!("{program}: unexpected argument '{other}'");
                print_usage(&program);
                return ExitCode::FAILURE;
            }
        }
    }

    let direction = match direction {
        Some(d) => d,
        None => {
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
