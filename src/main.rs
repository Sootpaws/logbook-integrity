use std::fs;
use std::path::PathBuf;
use logbook_integrity::parse;

fn main() {
    match run(&mut std::env::args().skip(1)) {
        Ok(_) => (),
        Err(message) => {
            eprintln!("Error: {message}");
            std::process::exit(1);
        }
    }
}

fn run(args: &mut impl Iterator<Item = String>) -> Result<(), String> {
    // Get action to perform
    let action = args.next();

    // Get input paths
    let paths = expand_paths(args.map(PathBuf::from).collect())?;

    // Perform action
    match action.as_deref() {
        Some("help") => help(),
        Some("validate") => {
            let _ = parse::parse_files(paths)?;
            Ok(())
        }
        None => {
            println!("No action given");
            help()
        }
        _ => {
            println!("Unrecognized action");
            help()
        }
    }
}

/// Print the help message
fn help() -> Result<(), String> {
    println!("logbook-integrity v{}", env!("CARGO_PKG_VERSION"));
    println!("Usage: logbook-integrity <action> [paths...]");
    println!("Actions:");
    println!("    help - print this message");
    println!("    validate - read in the logbook files, checking them for metadata errors");
    Ok(())
}

/// Expand a series of paths by recursing into directories
fn expand_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    paths
        .into_iter()
        .map(|path| match fs::metadata(&path) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    expand_paths(
                        fs::read_dir(path)
                            .unwrap()
                            .map(|entry| entry.unwrap().path())
                            .collect(),
                    )
                } else {
                    Ok(vec![path])
                }
            }
            Err(error) => Err(format!("Could not read path {}: {}", path.display(), error)),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|paths| paths.into_iter().flatten().collect())
}
