use dirs::home_dir;
use std::{fs, io};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, exit};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oneliner-cli")]
#[command(about = "A simple CLI tool to store and retrieve oneliners", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Store {
        #[arg(help = "The oneliner to store")]
        oneliner: String,
    },
    
    Get {
        #[arg(help = "The search term to find oneliners")]
        search: String,
    },

    List,
}

fn is_xclip_installed() -> bool {
    Command::new("which")
        .arg("xclip")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_oneliners_file() -> String {
    match home_dir() {
        Some(path) => {
            return path.to_str().expect("Invalid home directory").to_owned() + "/.oneliners";
        },
        None => panic!("Unable to locate cmds storage file.")
    }
}

fn list_oneliners(file_path: &str) {
    let file = match fs::File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("No oneliners stored yet.");
            return;
        }
    };

    let reader = BufReader::new(file);
    let entries: Vec<String> = reader.lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .take(10)
        .collect();

    if entries.is_empty() {
        println!("No entries found.");
        return;
    }

    for (i, line) in entries.iter().enumerate() {
        println!("{}: {}", i + 1, line);
    }
}

fn line_exists_in_file(file_path: &str, search: &str) -> bool {
    let file = match fs::File::open(file_path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.trim() == search.trim() {
                return true;
            }
        }
    }
    false
}

fn store_oneliner(oneliner: &str, file_path: &str) {
    if oneliner.contains('\n') || oneliner.contains("\r\n") {
        println!("Error: That's not a oneliner! Multi-line snippets are not current supported. You entered:");
        print!("{}\n", oneliner);
        return;
    }

    if !line_exists_in_file(file_path, oneliner) {
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .expect("Failed to open oneliners file");

        writeln!(file, "{}", oneliner).expect("Failed to write oneliner");
        println!("Snippet stored successfully! [{}]", file_path);
    } else {
        println!("Snippet already present.");
    }
}

fn get_oneliner(search: &str, file_path: &str) -> Vec<String> {
    let file = match fs::File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("No oneliners stored yet.");
            return vec![];
        }
    };
    
    let reader = BufReader::new(file);
    let matches: Vec<String> = reader.lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty() && line.contains(search))
        .take(3)
        .collect();
    
    if matches.is_empty() {
        println!("No matches found.");
    }
    
    matches
}

fn copy_to_clipboard(text: &str) {
    let _ = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())
        });
    println!("Snippet copied to clipboard!");
}

// fn get_zsh_completions_file() -> String {
//     match home_dir() {
//         Some(path) => {
//             return path.to_str().expect("Invalid home directory").to_owned() + "/.oh-my-zsh/completions";
//         },
//         None => panic!("Unable to locate .oh-my-zsh/completions file.")
//     }
// }

// fn store_in_zsh_completions(oneliner: &str) {
//     println!("Storing in zsh completions...");
//     let zsh_completions_path = get_zsh_completions_file();
//     store_oneliner(oneliner, &zsh_completions_path);
// }

fn main() {
    if !is_xclip_installed() {
        println!("xclip is not installed.");
        exit(1);
    }

    let cli = Cli::parse();

    let oneliners_file: String = get_oneliners_file();

    match cli.command {
        Commands::Store { oneliner } => {
            store_oneliner(&oneliner, &oneliners_file);
        },
        Commands::Get { search } => {
            let oneliners = get_oneliner(&search, &oneliners_file);
            for (i, oneliner) in oneliners.iter().enumerate() {
                println!("{}: {}", i + 1, oneliner);
            }

            if !oneliners.is_empty() {
                println!("Select a oneliner (1-{}):", oneliners.len());
                let mut selection = String::new();
                io::stdin().read_line(&mut selection).expect("Failed to read input");
                if let Ok(choice) = selection.trim().parse::<usize>() {
                    if choice > 0 && choice <= oneliners.len() {
                        copy_to_clipboard(&oneliners[choice - 1]);
                    }
                }
            }
        },
        Commands::List => list_oneliners(&oneliners_file),
    }
}
