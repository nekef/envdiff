use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the target environment file (e.g., .env)
    #[arg(short, long, default_value = ".env")]
    env: String,

    /// Path to the template environment file (e.g., .env.example)
    #[arg(short, long, default_value = ".env.example")]
    template: String,
}

fn extract_keys<P: AsRef<Path>>(path: P) -> io::Result<HashSet<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut keys = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        // Ignore comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(index) = trimmed.find('=') {
            let key = trimmed[..index].trim().to_string();
            if !key.is_empty() {
                keys.insert(key);
            }
        }
    }
    Ok(keys)
}

fn main() {
    let args = Args::parse();

    let target_keys = match extract_keys(&args.env) {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("{} Failed to read {}: {}", "Error:".red().bold(), args.env, e);
            std::process::exit(1);
        }
    };

    let template_keys = match extract_keys(&args.template) {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("{} Failed to read {}: {}", "Error:".red().bold(), args.template, e);
            std::process::exit(1);
        }
    };

    let missing: Vec<&String> = template_keys.difference(&target_keys).collect();
    let extra: Vec<&String> = target_keys.difference(&template_keys).collect();

    if missing.is_empty() && extra.is_empty() {
        println!("{}", "✔ Your .env file is perfectly synced with the template!".green().bold());
        return;
    }

    if !missing.is_empty() {
        println!("\n{}", "⚠️  Missing keys (defined in template but not in .env):".yellow().bold());
        for key in missing {
            println!("  {} {}", "-".red(), key);
        }
    }

    if !extra.is_empty() {
        println!("\n{}", "ℹ️  Extra keys (defined in .env but not in template):".blue().bold());
        for key in extra {
            println!("  {} {}", "+".green(), key);
        }
    }
    
    std::process::exit(1);
}
