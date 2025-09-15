//! Bebion CLI interface

pub mod repl;
pub mod runner;

use bebion_core::BebionEngine;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "bebion")]
#[command(about = "Bebion JavaScript Runtime")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Input file to execute
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Enable debug mode
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a JavaScript file
    Run {
        /// JavaScript file to run
        file: PathBuf,
        
        /// Arguments to pass to the script
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Start interactive REPL
    Repl {
        /// Load a file before starting REPL
        #[arg(short, long)]
        load: Option<PathBuf>,
    },
    
    /// Show version information
    Version,
    
    /// Show engine information
    Info,
    
    /// Compile JavaScript to bytecode
    Compile {
        /// Input JavaScript file
        input: PathBuf,
        
        /// Output bytecode file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Pretty print the bytecode
        #[arg(short, long)]
        pretty: bool,
    },
    
    /// Package management
    Package {
        #[command(subcommand)]
        action: PackageAction,
    },
}

#[derive(Subcommand)]
pub enum PackageAction {
    /// Install a package
    Install {
        /// Package name
        name: String,
        
        /// Package version
        #[arg(short, long)]
        version: Option<String>,
    },
    
    /// Remove a package
    Remove {
        /// Package name
        name: String,
    },
    
    /// List installed packages
    List,
    
    /// Update packages
    Update,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self, engine: &mut BebionEngine) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Some(Commands::Run { file, args }) => {
                info!("Running file: {:?}", file);
                runner::run_file(engine, file, args)?;
            }
            
            Some(Commands::Repl { load }) => {
                info!("Starting REPL");
                if let Some(load_file) = load {
                    info!("Loading file: {:?}", load_file);
                    runner::run_file(engine, load_file, &[])?;
                }
                repl::start_repl(engine)?;
            }
            
            Some(Commands::Version) => {
                self.show_version();
            }
            
            Some(Commands::Info) => {
                self.show_info(engine);
            }
            
            Some(Commands::Compile { input, output, pretty }) => {
                info!("Compiling file: {:?}", input);
                runner::compile_file(engine, input, output.as_ref(), *pretty)?;
            }
            
            Some(Commands::Package { action }) => {
                self.handle_package_action(action)?;
            }
            
            None => {
                if let Some(file) = &self.file {
                    info!("Running file: {:?}", file);
                    runner::run_file(engine, file, &[])?;
                } else {
                    info!("Starting REPL");
                    repl::start_repl(engine)?;
                }
            }
        }
        
        Ok(())
    }

    fn show_version(&self) {
        println!("Bebion JavaScript Runtime v{}", env!("CARGO_PKG_VERSION"));
        println!("Built with Rust {}", env!("RUSTC_VERSION"));
    }

    fn show_info(&self, engine: &BebionEngine) {
        println!("Bebion JavaScript Runtime");
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("ECMAScript: 2024");
        println!("Architecture: {}", std::env::consts::ARCH);
        println!("Platform: {}", std::env::consts::OS);
        
        // Show GC stats
        let stats = engine.gc_stats();
        println!("\nGarbage Collector:");
        println!("  Total objects: {}", stats.total_objects);
        println!("  Young objects: {}", stats.young_objects);
        println!("  Old objects: {}", stats.old_objects);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Total collections: {}", stats.total_collections);
        println!("  Bytes allocated: {}", stats.bytes_allocated);
        println!("  Bytes freed: {}", stats.bytes_freed);
    }

    fn handle_package_action(&self, action: &PackageAction) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            PackageAction::Install { name, version } => {
                println!("Installing package: {}", name);
                if let Some(ver) = version {
                    println!("Version: {}", ver);
                }
            }
            
            PackageAction::Remove { name } => {
            }
            
            PackageAction::List => {
            }
            
            PackageAction::Update => {
            }
        }
        
        Ok(())
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}
