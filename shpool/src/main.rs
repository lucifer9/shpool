// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
/// Shpool is a session persistence tool that works simillarly to tmux, but
/// aims to provide a simpler user experience. See [the
/// README](https://github.com/shell-pool/shpool) for more
/// info.
use clap::Parser;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Extract config file path from command line arguments without full parsing
fn extract_config_file(args: &[String]) -> Option<String> {
    let mut i = 1; // Skip binary name
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config-file" => {
                if i + 1 < args.len() {
                    return Some(args[i + 1].clone());
                }
            }
            arg if arg.starts_with("--config-file=") => {
                return Some(arg.strip_prefix("--config-file=").unwrap().to_string());
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Resolve command aliases by checking the first command argument against configured aliases.
/// Returns modified command line arguments with the alias expanded.
fn resolve_aliases() -> anyhow::Result<Vec<String>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Ok(args);
    }
    
    // Extract config file path manually
    let config_file = extract_config_file(&args);
    
    // Load config to check for aliases
    let config_manager = match libshpool::config::Manager::new(config_file.as_deref()) {
        Ok(manager) => manager,
        Err(_) => return Ok(args), // If config loading fails, return original args
    };
    
    let config = config_manager.get();
    if let Some(aliases) = &config.aliases {
        // Find the command position (usually after binary name and global flags)
        let mut command_pos = None;
        let mut i = 1; // Skip binary name
        
        while i < args.len() {
            let arg = &args[i];
            
            // Skip global flags
            if arg.starts_with('-') {
                i += 1;
                // Skip flag values for flags that take arguments
                if matches!(arg.as_str(), "--log-file" | "-l" | "--socket" | "-s" | "--config-file" | "-c") {
                    i += 1;
                }
                continue;
            }
            
            // This should be the command
            command_pos = Some(i);
            break;
        }
        
        if let Some(pos) = command_pos {
            let command = &args[pos];
            if let Some(resolved_command) = aliases.get(command) {
                let mut new_args = args.clone();
                new_args[pos] = resolved_command.clone();
                return Ok(new_args);
            }
        }
    }
    
    Ok(args)
}

fn main() -> anyhow::Result<()> {
    // Resolve aliases first
    let resolved_args = resolve_aliases()?;
    
    // Parse the resolved arguments
    let args = match libshpool::Args::try_parse_from(&resolved_args) {
        Ok(args) => args,
        Err(e) => {
            // If alias resolution caused a parsing error, show the error normally
            // This handles help messages, version, and actual errors correctly
            if resolved_args != env::args().collect::<Vec<_>>() {
                // For help and version, clap exits with status 0, for errors it exits with 2
                // We can check the error kind to handle appropriately
                use clap::error::ErrorKind;
                match e.kind() {
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                        print!("{}", e);
                        std::process::exit(0);
                    }
                    _ => {
                        eprint!("{}", e);
                        std::process::exit(2);
                    }
                }
            } else {
                // Re-parse to get proper clap error handling
                libshpool::Args::parse()
            }
        }
    };

    if args.version() {
        println!("shpool {VERSION}");
        return Ok(());
    }

    libshpool::run(args, None)
}
