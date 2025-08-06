// Copyright 2024 Google LLC
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

use std::{env, io::{self, Write}, path::PathBuf};

use anyhow::Context;
use tracing::info;

use super::{attach, config, detach};

pub fn run(
    config_manager: config::Manager,
    name: String,
    force: bool,
    confirm: bool,
    ttl: Option<String>,
    cmd: Option<String>,
    socket: PathBuf,
) -> anyhow::Result<()> {
    info!("\n\n======================== STARTING SWITCH ============================\n\n");

    if name.is_empty() {
        eprintln!("blank session names are not allowed");
        return Ok(());
    }
    if name.contains(char::is_whitespace) {
        eprintln!("whitespace is not allowed in session names");
        return Ok(());
    }

    // Check if we're currently in a shpool session
    let current_session = env::var("SHPOOL_SESSION_NAME").ok();
    
    match &current_session {
        Some(current) if current == &name => {
            eprintln!("already in session '{name}'");
            return Ok(());
        }
        Some(current) => {
            // We're in a session and switching to a different one
            info!("switching from session '{}' to session '{}'", current, name);
            
            if confirm {
                eprint!("Switch from session '{current}' to '{name}'? [y/N] ");
                io::stdout().flush().ok();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).context("reading confirmation")?;
                let input = input.trim().to_lowercase();
                
                if input != "y" && input != "yes" {
                    eprintln!("switch cancelled");
                    return Ok(());
                }
            }
            
            eprintln!("detaching from session '{current}'...");
            detach::run(vec![], &socket).context("detaching from current session")?;
            
            eprintln!("attaching to session '{name}'...");
        }
        None => {
            // Not currently in a session, just attach to target
            info!("not currently in a session, attaching to '{}'", name);
            eprintln!("attaching to session '{name}'...");
        }
    }

    // Use the attach logic to connect to the target session
    attach::run(config_manager, name, force, ttl, cmd, socket)
        .context("attaching to target session")
}