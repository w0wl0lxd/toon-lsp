// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! TOON Language Server and CLI binary entry point.

use clap::Parser;
use tower_lsp::{LspService, Server};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use toon_lsp::cli::{check, decode, diagnose, encode, format, symbols, Cli, Command};
use toon_lsp::lsp::ToonLanguageServer;


#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing with verbosity level
    let log_level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact(),
        )
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(log_level.into())
                .from_env_lossy(),
        )
        .init();

    // Execute command or start LSP server
    match cli.command {
        Some(Command::Lsp) | None => {
            tracing::info!("Starting TOON Language Server");

            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(ToonLanguageServer::new);
            Server::new(stdin, stdout, socket).serve(service).await;
        }
        Some(Command::Encode(args)) => {
            if let Err(e) = encode::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(e.exit_code()));
            }
        }
        Some(Command::Decode(args)) => {
            if let Err(e) = decode::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(decode::error_exit_code(&e)));
            }
        }
        Some(Command::Check(args)) => {
            if let Err(e) = check::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(check::error_exit_code(&e)));
            }
        }
        Some(Command::Format(args)) => {
            if let Err(e) = format::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(format::error_exit_code(&e)));
            }
        }
        Some(Command::Symbols(args)) => {
            if let Err(e) = symbols::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(symbols::error_exit_code(&e)));
            }
        }
        Some(Command::Diagnose(args)) => {
            if let Err(e) = diagnose::execute(&args) {
                eprintln!("Error: {e}");
                std::process::exit(i32::from(diagnose::error_exit_code(&e)));
            }
        }
        #[allow(unreachable_patterns)]
        Some(_) => {
            eprintln!("Unknown command");
            std::process::exit(1);
        }
    }
}
