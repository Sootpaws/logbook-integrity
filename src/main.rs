use std::path::PathBuf;
use std::fs;

fn main() {
    match run(&mut std::env::args()) {
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
    let paths = expand_paths(args.map(
        |path_string| PathBuf::from(path_string)
    ).collect())?;

    Ok(())
}

/// Expand a series of paths by recursing into directories
fn expand_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    paths.into_iter().map(|path| {
        match fs::metadata(&path) {
            Ok(metadata) => if metadata.is_dir() {
                expand_paths(
                    fs::read_dir(path)
                        .unwrap()
                        .map(|entry| entry.unwrap().path())
                        .collect()
                )
            } else {
                Ok(vec![path])
            }
            Err(error) => Err(format!(
                "Could not read path {}: {}",
                path.display(),
                error
            )),
        }
    }).collect::<Result<Vec<_>, _>>()
        .map(|paths| paths.into_iter().flatten().collect())
}
