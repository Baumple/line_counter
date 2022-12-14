use std::{
    fs::{DirEntry, File},
    io::Read,
};

use clap::Parser;
use thiserror::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The path of the file or directory of which the lines should be counted
    file_path: String,

    /// Skip empty lines
    #[clap(short, takes_value = false)]
    skip_empty_lines: bool,

    /// Enable the recursive flag.
    /// line_counter will count lines in subdirectories recursively
    #[clap(short, long, takes_value = false)]
    recursive: bool,

    #[clap(short, long, takes_value = false)]
    count_chars: bool,

    /// To ignore files completely add a ".ignore.lc" file to the directory and write down the files that should be ignored.
    ignored: Vec<String>,
}

impl Args {
    /// Checks if a ".ignore.lc" file is within the directory, and adds them to the ignored_vec.
    fn with_ignored(mut self) -> Result<Self> {
        if !std::fs::metadata(&self.file_path)?.is_dir() {
            return Ok(self);
        }
        for entry in std::fs::read_dir(&self.file_path)?.flatten() {
            if entry.file_name() == ".lcignore" {
                let mut f = File::open(entry.path())?;

                let mut ignored = String::new();
                f.read_to_string(&mut ignored)?;

                self.ignored = ignored
                    .lines()
                    .map(|line| line.trim().to_string())
                    .collect();
                self.ignored.push(String::from(".lcignore"));
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("Error occurred while reading file")]
    LcIoError(#[from] std::io::Error),

    #[error("Error occurred while parsing file name")]
    FileNameError,
}

type Result<T> = std::result::Result<T, Error>;
fn main() -> Result<()> {
    let args = Args::parse().with_ignored()?;

    let file_metadata = std::fs::metadata(&args.file_path)?;

    let mut lines: usize = 0;
    let mut characters = 0;

    if file_metadata.is_dir() {
        (lines, characters) = get_dir_lines(&args.file_path, &args, 0)?;
    } else {
        let mut file = File::open(&args.file_path)?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;
        if args.skip_empty_lines {
            for line in buffer.lines() {
                if !line.trim().is_empty() {
                    lines += 1;
                    if args.count_chars {
                        characters += line.chars().count();
                    }
                }
            }
        } else {
            lines += buffer.lines().count();
            if args.count_chars {
                let _: Vec<_> = buffer
                    .lines()
                    .map(|x| characters += x.chars().count())
                    .collect();
            }
        }
    }

    println!(
        "\nTotal number of lines in {}: {}",
        if file_metadata.is_dir() {
            "directory"
        } else {
            "file"
        },
        lines
    );

    if args.count_chars {
        println!(
            "Total number of characters in {}: {}",
            if file_metadata.is_dir() {
                "directory"
            } else {
                "file"
            },
            characters
        );
    }

    Ok(())
}

fn get_dir_lines(file_path: &str, args: &Args, depth: usize) -> Result<(usize, usize)> {
    let mut lines = 0;
    let mut characters = 0;
    let mut indenting = String::new();

    let mut maybe_dirs: Vec<DirEntry> = Vec::new();

    for _d in 0..depth {
        indenting += "  ";
    }

    println!("{}{}:", indenting, file_path);
    'outer: for entry in std::fs::read_dir(&file_path)?.flatten() {
        // check if file should be ignored
        let file_name = entry
            .file_name()
            .to_str()
            .ok_or(Error::FileNameError)?
            .to_string();
        for ignored in &args.ignored {
            if file_name == *ignored {
                continue 'outer;
            }
        }

        if entry.metadata()?.is_dir() {
            if args.recursive {
                maybe_dirs.push(entry);
            }
            continue;
        }

        let mut file = File::open(entry.path())?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        if args.skip_empty_lines {
            for line in buffer.lines() {
                if !line.trim().is_empty() {
                    lines += 1;
                    if args.count_chars {
                        characters += line.chars().count();
                    }
                }
            }
        } else {
            lines += buffer.lines().count();
            if args.count_chars {
                let _: Vec<_> = buffer
                    .lines()
                    .map(|line| characters += line.chars().count())
                    .collect();
            }
        }

        dbg!(characters);
        println!(
            "{}> {}: {}",
            indenting,
            entry.file_name().to_str().ok_or(Error::FileNameError)?,
            lines
        );
        if lines == 69 {
            println!("  NICE!");
        }
    }
    for dir in maybe_dirs {
        let (tmp_lines, tmp_characters) = get_dir_lines(
            dir.path().to_str().ok_or(Error::FileNameError)?,
            args,
            depth + 1,
        )?;

        lines += tmp_lines;
        characters += tmp_characters;
    }

    Ok((lines, characters))
}
