use std::io::{Read, Write};
use std::process::ExitCode;

const USAGE: &str = "\
Convert HL7 v2.5 messages (pipe-delimited ER7) to HL7 v2.xml.

Usage: hl7_v2_from_er7_into_xml [OPTIONS] [FILE]

Arguments:
  [FILE]  Input file holding one or more ER7 messages; \"-\" or omitted reads stdin

Options:
  -o, --output <FILE>  Write XML to FILE instead of stdout
      --flat           Do not group segments into message-structure groups
  -h, --help           Print help
  -V, --version        Print version";

fn main() -> ExitCode {
    let mut flat = false;
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
                println!("hl7_v2_from_er7_into_xml {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--flat" => flat = true,
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

    let messages = hl7_v2_from_er7_into_xml::split_messages(&text);
    if messages.is_empty() {
        return fail("no HL7 message found in input");
    }
    let options = hl7_v2_from_er7_into_xml::Options { flat };
    let mut documents = Vec::with_capacity(messages.len());
    for (i, message) in messages.iter().enumerate() {
        match hl7_v2_from_er7_into_xml::convert_with_options(message, options) {
            Ok(xml) => documents.push(xml),
            Err(e) => return fail(&format!("message {}: {e}", i + 1)),
        }
    }
    let out_text = documents.join("\n");

    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, out_text) {
                return fail(&format!("writing {path}: {e}"));
            }
        }
        None => {
            if std::io::stdout().write_all(out_text.as_bytes()).is_err() {
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("hl7_v2_from_er7_into_xml: error: {message}");
    ExitCode::FAILURE
}
