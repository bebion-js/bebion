//! Bebion JavaScript Runtime
//! 
//! A high-performance JavaScript runtime built with Rust and C.
//! Provides ECMAScript 2024 compliance with advanced features.

use bebion_cli::Cli;
use bebion_core::BebionEngine;
use tracing::{info, Level};
use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Bebion JavaScript Runtime v{}", env!("CARGO_PKG_VERSION"));

    // Initialize the core engine
    let mut engine = BebionEngine::new()?;
    
    // Start the CLI
    let cli = Cli::new();
    cli.run(&mut engine)?;

    Ok(())
}