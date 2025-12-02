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

//! TOON Language Server binary entry point.

use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use toon_lsp::lsp::ToonLanguageServer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting TOON Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(ToonLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
