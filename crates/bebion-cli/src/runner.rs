//! File execution and compilation

use bebion_core::{BebionEngine, BebionError};
use bebion_compiler::bytecode::Bytecode;
use colored::*;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, error, info};

pub fn run_file(
    engine: &mut BebionEngine,
    file_path: &Path,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running file: {:?}", file_path);
    
    // Check if file exists
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    // Read the file
    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    debug!("Read {} bytes from {}", source.len(), file_path.display());

    // Execute the script
    let start_time = Instant::now();
    
    match engine.execute_script(&source) {
        Ok(_result) => {
            let duration = start_time.elapsed();
            debug!("Script executed successfully in {:?}", duration);
            Ok(())
        }
        Err(err) => {
            print_execution_error(&err, file_path);
            std::process::exit(1);
        }
    }
}

pub fn compile_file(
    engine: &mut BebionEngine,
    input_path: &Path,
    output_path: Option<&PathBuf>,
    pretty: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Compiling file: {:?}", input_path);
    
    // Check if input file exists
    if !input_path.exists() {
        return Err(format!("File not found: {}", input_path.display()).into());
    }

    // Read the source file
    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;

    // Parse the source
    let mut parser = bebion_parser::Parser::new();
    let ast = parser.parse(&source)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Compile to bytecode
    let mut compiler = bebion_compiler::Compiler::new();
    let bytecode = compiler.compile(&ast)
        .map_err(|e| format!("Compile error: {}", e))?;

    // Determine output path
    let output_file = if let Some(path) = output_path {
        path.clone()
    } else {
        let mut path = input_path.to_path_buf();
        path.set_extension("bbc"); // Bebion Bytecode
        path
    };

    // Serialize bytecode
    let serialized = if pretty {
        serde_json::to_string_pretty(&bytecode)?
    } else {
        serde_json::to_string(&bytecode)?
    };

    // Write to output file
    fs::write(&output_file, serialized)
        .map_err(|e| format!("Failed to write output file {}: {}", output_file.display(), e))?;

    println!(
        "{} Compiled {} to {}",
        "✓".green().bold(),
        input_path.display(),
        output_file.display()
    );

    // Show compilation stats
    println!("  Instructions: {}", bytecode.instructions.len());
    println!("  Constants: {}", bytecode.constants.len());
    println!("  Names: {}", bytecode.names.len());
    println!("  Size: {} bytes", serialized.len());

    Ok(())
}

pub fn run_bytecode_file(
    engine: &mut BebionEngine,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running bytecode file: {:?}", file_path);
    
    // Check if file exists
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    // Read the bytecode file
    let bytecode_json = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    // Deserialize bytecode
    let bytecode: Bytecode = serde_json::from_str(&bytecode_json)
        .map_err(|e| format!("Failed to parse bytecode: {}", e))?;

    debug!("Loaded bytecode with {} instructions", bytecode.instructions.len());

    // Execute the bytecode
    let start_time = Instant::now();
    
    match engine.execute_bytecode(&bytecode) {
        Ok(_result) => {
            let duration = start_time.elapsed();
            debug!("Bytecode executed successfully in {:?}", duration);
            Ok(())
        }
        Err(err) => {
            print_execution_error(&err, file_path);
            std::process::exit(1);
        }
    }
}

fn print_execution_error(error: &BebionError, file_path: &Path) {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    match error {
        BebionError::ParseError(msg) => {
            eprintln!("{}: {} in {}", 
                "SyntaxError".red().bold(), 
                msg, 
                file_name.yellow()
            );
        }
        BebionError::CompileError(msg) => {
            eprintln!("{}: {} in {}", 
                "CompileError".red().bold(), 
                msg, 
                file_name.yellow()
            );
        }
        BebionError::RuntimeError(msg) => {
            eprintln!("{}: {} in {}", 
                "RuntimeError".red().bold(), 
                msg, 
                file_name.yellow()
            );
        }
        BebionError::ModuleError(msg) => {
            eprintln!("{}: {} in {}", 
                "ModuleError".red().bold(), 
                msg, 
                file_name.yellow()
            );
        }
    }
}

pub fn benchmark_file(
    engine: &mut BebionEngine,
    file_path: &Path,
    iterations: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Benchmarking file: {:?} ({} iterations)", file_path, iterations);
    
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    let mut total_time = std::time::Duration::new(0, 0);
    let mut successful_runs = 0;

    println!("Running benchmark...");
    
    for i in 1..=iterations {
        let start_time = Instant::now();
        
        match engine.execute_script(&source) {
            Ok(_) => {
                let duration = start_time.elapsed();
                total_time += duration;
                successful_runs += 1;
                
                if i % (iterations / 10).max(1) == 0 {
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap_or(());
                }
            }
            Err(err) => {
                eprintln!("\nError in iteration {}: {}", i, err);
                break;
            }
        }
        
        // Force garbage collection between runs for consistent measurements
        engine.gc_collect();
    }
    
    println!();
    
    if successful_runs > 0 {
        let avg_time = total_time / successful_runs as u32;
        let ops_per_sec = 1.0 / avg_time.as_secs_f64();
        
        println!("{}", "Benchmark Results:".bright_blue().bold());
        println!("  Successful runs: {}/{}", successful_runs, iterations);
        println!("  Total time: {:?}", total_time);
        println!("  Average time: {:?}", avg_time);
        println!("  Operations/sec: {:.2}", ops_per_sec);
        
        // Show GC stats
        let stats = engine.gc_stats();
        println!("  GC collections: {}", stats.total_collections);
        println!("  Memory freed: {} bytes", stats.bytes_freed);
    } else {
        eprintln!("No successful runs completed");
    }

    Ok(())
}

pub fn analyze_file(
    engine: &mut BebionEngine,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Analyzing file: {:?}", file_path);
    
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    // Parse the source
    let mut parser = bebion_parser::Parser::new();
    let ast = parser.parse(&source)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Compile to bytecode
    let mut compiler = bebion_compiler::Compiler::new();
    let bytecode = compiler.compile(&ast)
        .map_err(|e| format!("Compile error: {}", e))?;

    println!("{}", "File Analysis:".bright_blue().bold());
    println!("  File: {}", file_path.display());
    println!("  Size: {} bytes", source.len());
    println!("  Lines: {}", source.lines().count());
    
    println!("\n{}", "AST Analysis:".bright_blue().bold());
    println!("  Nodes: {}", ast.node_count());
    
    println!("\n{}", "Bytecode Analysis:".bright_blue().bold());
    println!("  Instructions: {}", bytecode.instructions.len());
    println!("  Constants: {}", bytecode.constants.len());
    println!("  Names: {}", bytecode.names.len());
    
    // Analyze instruction distribution
    let mut instruction_counts = std::collections::HashMap::new();
    for instruction in &bytecode.instructions {
        let name = format!("{:?}", instruction).split('(').next().unwrap_or("Unknown").to_string();
        *instruction_counts.entry(name).or_insert(0) += 1;
    }
    
    println!("\n{}", "Instruction Distribution:".bright_blue().bold());
    let mut sorted_instructions: Vec<_> = instruction_counts.iter().collect();
    sorted_instructions.sort_by(|a, b| b.1.cmp(a.1));
    
    for (instruction, count) in sorted_instructions.iter().take(10) {
        let percentage = (*count as f64 / bytecode.instructions.len() as f64) * 100.0;
        println!("  {}: {} ({:.1}%)", instruction, count, percentage);
    }

    Ok(())
}
