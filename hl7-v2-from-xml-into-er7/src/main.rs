use std::io::{Read, Write};
use std::process::ExitCode;

const USAGE: &str = "\
Convert HL7 v2.xml XML documents back to HL7 v2.5 ER7 (pipe-delimited).

Usage: hl7_v2_from_xml_into_er7 [OPTIONS] [FILE]

Arguments:
  [FILE]  Input file holding one v2.xml document; \"-\" or omitted reads stdin

Options:
  -o, --output <FILE>        Write ER7 to FILE instead of stdout
  -t, --terminator <KIND>    Segment terminator to write: cr (default), lf, crlf
      --trailing-terminator  End the last segment with a terminator too
  -h, --help                 Print help
  -V, --version               Print version";

fn main() -> ExitCode {
    let mut terminator = er7::Terminator::Cr;
    let mut trailing_terminator = false;
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("hl7_v2_from_xml_into_er7 {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--trailing-terminator" => trailing_terminator = true,
            "-t" | "--terminator" => match args.next().as_deref() {
                Some("cr") => terminator = er7::Terminator::Cr,
                Some("lf") => terminator = er7::Terminator::Lf,
                Some("crlf") => terminator = er7::Terminator::CrLf,
                Some(other) => return fail(&format!("unknown terminator: {other}")),
                None => return fail("missing value for --terminator"),
            },
            "-o" | "--output" => match args.next() {
                Some(path) => output = Some(path),
                None => return fail("missing value for --output"),
            },
            "-" => input = Some("-".to_string()),
            _ if arg.starts_with('-') => return fail(&format!("unknown option: {arg}")),
            _ => {
                if input.is_some() {
                    return fail("more than one input file given");
                }
                input = Some(arg);
            }
        }
    }

    let text = match input.as_deref() {
        None | Some("-") => {
            let mut buffer = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                return fail(&format!("reading stdin: {e}"));
            }
            buffer
        }
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return fail(&format!("reading {path}: {e}")),
        },
    };

    let options = er7::RenderOptions {
        terminator,
        trailing_terminator,
    };
    let er7_text = match hl7_v2_from_xml_into_er7::convert_with_options(&text, options) {
        Ok(er7_text) => er7_text,
        Err(e) => return fail(&e.to_string()),
    };

    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, er7_text) {
                return fail(&format!("writing {path}: {e}"));
            }
        }
        None => {
            if std::io::stdout().write_all(er7_text.as_bytes()).is_err() {
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("hl7_v2_from_xml_into_er7: error: {message}");
    ExitCode::FAILURE
}
