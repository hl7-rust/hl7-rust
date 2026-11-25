// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::process::ExitCode;

const USAGE: &str = "\
Convert HL7 v2.5 messages (pipe-delimited ER7) to JSON.

Usage: hl7-2-from-er7-into-json [OPTIONS] [FILE]

Arguments:
  [FILE]  Input file holding one or more ER7 messages; \"-\" or omitted reads stdin

Options:
  -o, --output <FILE>  Write JSON to FILE instead of stdout
      --flat           Do not group segments into message-structure groups
      --require-order  Fail when the JSON cannot keep the segments in their original order
      --compact        Emit compact (single-line) JSON instead of pretty-printed
  -h, --help           Print help
  -V, --version        Print version";

fn main() -> ExitCode {
    let mut flat = false;
    let mut compact = false;
    let mut require_order = false;
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
                println!("hl7-2-from-er7-into-json {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--flat" => flat = true,
            "--compact" => compact = true,
            "--require-order" => require_order = true,
            "-o" | "--output" => match args.next() {
                Some(path) => output = Some(path),
                None => return fail("missing value for --output"),
            },
            // Guarded like any other input argument: without the check, a
            // second input given as "-" would silently replace the first
            // and read standard input instead of the named file.
            "-" => {
                if input.is_some() {
                    return fail("more than one input file given");
                }
                input = Some("-".to_string());
            }
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

    let messages = hl7_2_from_er7_into_json::split_messages(&text);
    if messages.is_empty() {
        return fail("no HL7 message found in input");
    }
    let options = hl7_2_from_er7_into_json::Options { flat, compact };
    let mut documents = Vec::with_capacity(messages.len());
    for (i, message) in messages.iter().enumerate() {
        // An `Err` here would be the same error `convert_with_options` reports
        // just below, so only a definite "order is lost" is handled here.
        if require_order
            && matches!(
                hl7_2_from_er7_into_json::preserves_segment_order(message, options),
                Ok(false)
            )
        {
            return fail(&format!(
                "message {}: the JSON cannot keep its segments in their original order \
                 (same-named segments are not adjacent and the message is not grouped); \
                 keep the ER7 for replay, or drop --require-order",
                i + 1
            ));
        }
        match hl7_2_from_er7_into_json::convert_with_options(message, options) {
            Ok(json) => documents.push(json),
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
    eprintln!("hl7-2-from-er7-into-json: error: {message}");
    ExitCode::FAILURE
}
