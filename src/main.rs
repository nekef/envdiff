use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

fn main() {
    // Process positional arguments for our features
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--export-markdown" => {
                if let Err(_) = dotenvy::dotenv() {
                    eprintln!("❌ Error: No local .env file found in the current directory.");
                    process::exit(1);
                }
                export_env_as_markdown();
                return;
            }
            "--check" => {
                if args.len() < 3 {
                    eprintln!("❌ Error: Please provide a template file path to check against.");
                    eprintln!("   Usage: envdiff --check <template_file_path>");
                    process::exit(1);
                }
                if let Err(_) = dotenvy::dotenv() {
                    eprintln!("⚠️  Warning: No local .env file found. Checking system variables instead.\n");
                }
                if let Err(e) = verify_against_template(&args[2]) {
                    eprintln!("❌ Error reading template file: {e}");
                    process::exit(1);
                }
                return;
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {
                eprintln!("❌ Error: Unknown argument '{}'", args[1]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Default action: Try to load the .env file.
    // If it's not found, stop immediately and tell the user instead of dumping system profiles.
    if let Err(_) = dotenvy::dotenv() {
        println!("❌ Error: No local .env file found in the current directory.");
        println!("   Please create a .env file or use '--help' to see available options.");
        process::exit(1);
    }

    println!("=== Current Environment Configuration (From .env) ===");
    
    // Read the file manually to strictly print ONLY what the user explicitly put in their .env
    if let Ok(file) = File::open(".env") {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                let trimmed = line_str.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, _)) = trimmed.split_once('=') {
                    let key = key.trim();
                    // Pull the parsed value from env (which dotenvy loaded for us)
                    if let Ok(value) = env::var(key) {
                        let display_value = get_secure_env_value(key, value);
                        println!("{key}={display_value}");
                    }
                }
            }
        }
    }
}

/// Feature: Print helpful navigation details
fn print_usage() {
    println!("envdiff - Enhanced environment configuration utility\n");
    println!("Usage:");
    println!("  envdiff                     Show configuration keys from your local .env");
    println!("  envdiff --export-markdown   Generate a scannable markdown table blueprint");
    println!("  envdiff --check <file>      Verify current context matches an example layout");
    println!("  envdiff --help, -h          Display this menu configuration context");
}

/// Feature: Automatically obscure known keys to prevent secret leaks to stdout logs
fn get_secure_env_value(key: &str, value: String) -> String {
    let sensitive_keys = ["SECRET", "PASSWORD", "TOKEN", "KEY", "AUTH", "DATABASE_URL"];

    let is_sensitive = sensitive_keys.iter().any(|&keyword| {
        key.to_uppercase().contains(keyword)
    });

    if is_sensitive && !value.is_empty() {
        format!("[REDACTED (Length: {})]", value.len())
    } else {
        value
    }
}

/// Feature: Export explicit .env definitions formatting directly to markdown documentation strings
fn export_env_as_markdown() {
    println!("| Environment Variable | Value |");
    println!("|----------------------|-------|");
    
    if let Ok(file) = File::open(".env") {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                let trimmed = line_str.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, _)) = trimmed.split_once('=') {
                    let key = key.trim();
                    if let Ok(value) = env::var(key) {
                        let clean_val = value.replace("|", "\\|");
                        let display_val = get_secure_env_value(key, clean_val);
                        println!("| `{}` | {} |", key, display_val);
                    }
                }
            }
        }
    }
}

/// Feature: Parse a target template file (like .env.example) and cross-reference keys
fn verify_against_template<P: AsRef<Path>>(example_path: P) -> io::Result<()> {
    let file = File::open(example_path)?;
    let reader = io::BufReader::new(file);

    println!("Checking environment state against configuration design layout...");
    let mut missing_count = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.trim();
            if env::var(key).is_err() {
                println!("⚠️  Missing key structural alignment: `{key}` is defined in template but absent!");
                missing_count += 1;
            }
        }
    }

    if missing_count == 0 {
        println!("✅ Success: Local environment matches template expectations perfectly.");
    } else {
        println!("\n❌ Validation failed: {missing_count} fields missing from system execution state.");
    }

    Ok(())
}
