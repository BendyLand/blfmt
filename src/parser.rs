use std::env;
use std::path::Path;

use crate::{options, utils};

#[derive(Clone, Debug)]
pub struct ParsedArgs {
    pub filepath: String,
    pub is_stdin: bool,
    pub write: bool,
    pub help: bool,
    pub style: utils::Style,
    pub txt_opts: options::TxtOpts,
    pub titles: Vec<String>,
}

fn is_known_flag(arg: &str) -> bool {
    matches!(
        arg,
        "-h" | "--help"
            | "-w"
            | "--write"
            | "-hw"
            | "-wh"
            | "-"
            | "--stdin"
            | "-s"
            | "--style"
            | "-o"
            | "--opts"
            | "--options"
            | "-t"
            | "--titles"
            | "--"
    ) || arg.starts_with("-s=")
        || arg.starts_with("--style=")
        || arg.starts_with("--stdin=")
        || arg.starts_with("-o=")
        || arg.starts_with("--opts=")
        || arg.starts_with("--options=")
}

fn looks_like_filepath_candidate(s: &str) -> bool {
    Path::new(s).is_file() || utils::check_valid_file_ext(s)
}

fn normalize_ext(ext: &str) -> String {
    let trimmed = ext.trim();
    if trimmed.starts_with('.') {
        trimmed.to_string()
    } else {
        format!(".{}", trimmed)
    }
}

pub fn parse_args_from<I, S>(raw_args: I) -> Result<ParsedArgs, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = raw_args.into_iter().map(Into::into).collect();
    if args.is_empty() {
        return Err(String::new());
    }

    let mut positionals: Vec<String> = Vec::new();
    let mut stdin_ext_inline: Option<String> = None;
    let mut is_stdin = false;
    let mut write = false;
    let mut help = false;
    let mut style = utils::Style::Stroustrup;
    let mut txt_opts = options::TxtOpts::default();
    let mut titles: Vec<String> = Vec::new();
    let mut saw_titles_flag = false;
    let mut titles_came_last = false;
    let mut after_double_dash = false;

    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i].as_str();

        if after_double_dash {
            positionals.push(arg.to_string());
            titles_came_last = false;
            i += 1;
            continue;
        }

        match arg {
            "--" => {
                after_double_dash = true;
                titles_came_last = false;
                i += 1;
            },
            "-h" | "--help" => {
                help = true;
                i += 1;
            },
            "-w" | "--write" => {
                write = true;
                titles_came_last = false;
                i += 1;
            },
            "-hw" | "-wh" => {
                help = true;
                write = true;
                i += 1;
            },
            "-" | "--stdin" => {
                is_stdin = true;
                titles_came_last = false;
                i += 1;
            },
            _ if arg.starts_with("--stdin=") => {
                is_stdin = true;
                let val = &arg["--stdin=".len()..];
                if val.is_empty() {
                    return Err("Error: --stdin= requires a file extension.".to_string());
                }
                stdin_ext_inline = Some(val.to_string());
                titles_came_last = false;
                i += 1;
            },
            "-s" | "--style" => {
                if i + 1 >= args.len() {
                    return Err(format!("Error: '{}' requires a style name (allman, knr, stroustrup).", arg));
                }
                let val = &args[i + 1];
                let Some(parsed_style) = options::parse_c_style_value(val) else {
                    return Err(format!(
                        "Error: invalid style '{}'. Expected one of: allman, knr, stroustrup.",
                        val
                    ));
                };
                style = parsed_style;
                titles_came_last = false;
                i += 2;
            },
            _ if arg.starts_with("-s=") || arg.starts_with("--style=") => {
                let val = arg.split_once('=').map(|(_, v)| v).unwrap_or("");
                let Some(parsed_style) = options::parse_c_style_value(val) else {
                    return Err(format!(
                        "Error: invalid style '{}'. Expected one of: allman, knr, stroustrup.",
                        val
                    ));
                };
                style = parsed_style;
                titles_came_last = false;
                i += 1;
            },
            "-o" | "--opts" | "--options" => {
                if i + 2 >= args.len() {
                    return Err(format!(
                        "Error: '{}' requires two numeric arguments: <columns> <spacing>.",
                        arg
                    ));
                }
                let cols_str = &args[i + 1];
                let spacing_str = &args[i + 2];
                let cols = cols_str.parse::<usize>().map_err(|_| {
                    format!("Error: invalid columns value '{}' for '{}' (expected a non-negative integer).", cols_str, arg)
                })?;
                let spacing = spacing_str.parse::<usize>().map_err(|_| {
                    format!("Error: invalid spacing value '{}' for '{}' (expected a non-negative integer).", spacing_str, arg)
                })?;
                txt_opts = options::TxtOpts {
                    columns: cols,
                    spacing,
                };
                titles_came_last = false;
                i += 3;
            },
            _ if arg.starts_with("-o=") || arg.starts_with("--opts=") || arg.starts_with("--options=") => {
                let val = arg.split_once('=').map(|(_, v)| v).unwrap_or("");
                let parts: Vec<&str> = val.split(|c| c == ',' || c == ':').filter(|s| !s.is_empty()).collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Error: '{}' requires two numeric values (e.g. --opts=80,1).",
                        arg
                    ));
                }
                let cols = parts[0].parse::<usize>().map_err(|_| {
                    format!("Error: invalid columns value '{}' in '{}'.", parts[0], arg)
                })?;
                let spacing = parts[1].parse::<usize>().map_err(|_| {
                    format!("Error: invalid spacing value '{}' in '{}'.", parts[1], arg)
                })?;
                txt_opts = options::TxtOpts {
                    columns: cols,
                    spacing,
                };
                titles_came_last = false;
                i += 1;
            },
            "-t" | "--titles" => {
                saw_titles_flag = true;
                i += 1;
                let start_len = titles.len();
                while i < args.len() && !is_known_flag(&args[i]) {
                    titles.push(args[i].clone());
                    i += 1;
                }
                if titles.len() == start_len && !help {
                    return Err(format!("Error: '{}' requires at least one title string.", arg));
                }
                titles_came_last = i == args.len();
            },
            _ if arg.starts_with('-') => {
                if args.iter().any(|a| a == "-h" || a == "--help" || a == "-hw" || a == "-wh") {
                    help = true;
                    break;
                }
                return Err(format!("Error: unknown option '{}'.", arg));
            },
            _ => {
                positionals.push(arg.to_string());
                titles_came_last = false;
                i += 1;
            },
        }
    }

    if help {
        return Ok(ParsedArgs {
            filepath: String::new(),
            is_stdin: false,
            write: false,
            help: true,
            style,
            txt_opts,
            titles,
        });
    }

    // If `-t ... <file>` was placed before any standalone positional target, recover `<file>` from the end of `titles`
    if positionals.is_empty() && stdin_ext_inline.is_none() && saw_titles_flag && !titles.is_empty() {
        let should_pop = if is_stdin {
            titles.len() >= 2 && titles_came_last
        } else {
            titles.last().map(|s| looks_like_filepath_candidate(s)).unwrap_or(false)
        };
        if should_pop {
            if let Some(candidate) = titles.pop() {
                if titles.is_empty() {
                    return Err("Error: '-t'/'--titles' requires at least one title before the file path.".to_string());
                }
                positionals.push(candidate);
            }
        }
    }

    let target = if let Some(ext) = stdin_ext_inline {
        if !positionals.is_empty() {
            return Err(format!(
                "Error: unexpected extra argument '{}' when '--stdin=' is already specified.",
                positionals[0]
            ));
        }
        normalize_ext(&ext)
    } else {
        if positionals.is_empty() {
            if is_stdin {
                return Err("Error: stdin mode ('-' or '--stdin') requires a file extension (e.g. 'c' or '.cpp').".to_string());
            } else {
                return Err("Error: missing file path.".to_string());
            }
        }
        if positionals.len() > 1 {
            return Err(format!(
                "Error: unexpected extra positional argument '{}' (expected a single target).",
                positionals[1]
            ));
        }
        if is_stdin {
            normalize_ext(&positionals[0])
        } else {
            positionals.remove(0)
        }
    };

    Ok(ParsedArgs {
        filepath: target,
        is_stdin,
        write: write && !is_stdin,
        help: false,
        style,
        txt_opts,
        titles,
    })
}

pub fn parse_args() -> Option<ParsedArgs> {
    let raw: Vec<String> = env::args().skip(1).collect();
    match parse_args_from(raw) {
        Ok(parsed) => Some(parsed),
        Err(msg) => {
            if !msg.is_empty() {
                eprintln!("{}", msg);
            }
            utils::print_usage();
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_and_style_in_any_order() {
        let a = parse_args_from(["foo.c", "-s", "allman", "-w"]).unwrap();
        assert_eq!(a.filepath, "foo.c");
        assert!(matches!(a.style, utils::Style::Allman));
        assert!(a.write);
        assert!(!a.is_stdin);

        let b = parse_args_from(["-w", "--style", "knr", "foo.c"]).unwrap();
        assert_eq!(b.filepath, "foo.c");
        assert!(matches!(b.style, utils::Style::KnR));
        assert!(b.write);

        let c = parse_args_from(["--style=allman", "foo.cpp", "--write"]).unwrap();
        assert_eq!(c.filepath, "foo.cpp");
        assert!(matches!(c.style, utils::Style::Allman));
        assert!(c.write);
    }

    #[test]
    fn parses_stdin_in_any_order_preserving_other_flags() {
        let a = parse_args_from(["-", "c", "-s", "allman"]).unwrap();
        assert!(a.is_stdin);
        assert_eq!(a.filepath, ".c");
        assert!(matches!(a.style, utils::Style::Allman));
        assert!(!a.write);

        let b = parse_args_from(["-s", "knr", "cpp", "--stdin", "-w"]).unwrap();
        assert!(b.is_stdin);
        assert_eq!(b.filepath, ".cpp");
        assert!(matches!(b.style, utils::Style::KnR));
        assert!(!b.write); // write suppressed in stdin mode

        let c = parse_args_from(["--stdin=.h", "-s=allman"]).unwrap();
        assert!(c.is_stdin);
        assert_eq!(c.filepath, ".h");
        assert!(matches!(c.style, utils::Style::Allman));
    }

    #[test]
    fn parses_txt_opts_and_titles_in_any_order() {
        let a = parse_args_from(["notes.txt", "-o", "72", "2", "-t", "Intro", "Summary", "-w"]).unwrap();
        assert_eq!(a.filepath, "notes.txt");
        assert_eq!(a.txt_opts, options::TxtOpts { columns: 72, spacing: 2 });
        assert_eq!(a.titles, vec!["Intro", "Summary"]);
        assert!(a.write);

        let b = parse_args_from(["-t", "Intro", "Summary", "--options", "60", "3", "notes.txt"]).unwrap();
        assert_eq!(b.filepath, "notes.txt");
        assert_eq!(b.txt_opts, options::TxtOpts { columns: 60, spacing: 3 });
        assert_eq!(b.titles, vec!["Intro", "Summary"]);

        let c = parse_args_from(["--opts", "100", "1", "-t", "Part 1", "Part 2", "notes.txt"]).unwrap();
        assert_eq!(c.filepath, "notes.txt");
        assert_eq!(c.txt_opts, options::TxtOpts { columns: 100, spacing: 1 });
        assert_eq!(c.titles, vec!["Part 1", "Part 2"]);
    }

    #[test]
    fn parses_help_anywhere_and_rejects_bad_flags() {
        let h1 = parse_args_from(["-h"]).unwrap();
        assert!(h1.help);
        let h2 = parse_args_from(["-s", "allman", "foo.c", "--help"]).unwrap();
        assert!(h2.help);

        assert!(parse_args_from(["-s", "foo.c"]).is_err());
        assert!(parse_args_from(["-o", "80", "notes.txt"]).is_err());
        assert!(parse_args_from(["--unknown", "foo.c"]).is_err());
        assert!(parse_args_from(["a.c", "b.c"]).is_err());
    }
}
