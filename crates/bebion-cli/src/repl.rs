//! Interactive REPL (Read-Eval-Print Loop)

use bebion_core::{BebionEngine, BebionError};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::io::{self, Write};
use tracing::{debug, error};

pub fn start_repl(engine: &mut BebionEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Bebion JavaScript Runtime".bright_blue().bold());
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Type {} for help, {} to exit", ".help".yellow(), ".exit".yellow());
    println!();

    let mut rl = DefaultEditor::new()?;
    let mut line_number = 1;
    let mut multiline_buffer = String::new();
    let mut in_multiline = false;

    loop {
        let prompt = if in_multiline {
            format!("{}> ", "...".bright_black())
        } else {
            format!("{}> ", format!("bebion:{}", line_number).bright_green())
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                
                // Handle REPL commands
                if !in_multiline && trimmed.starts_with('.') {
                    match handle_repl_command(trimmed, engine) {
                        ReplCommand::Exit => break,
                        ReplCommand::Continue => continue,
                        ReplCommand::Error(msg) => {
                            println!("{}: {}", "Error".red().bold(), msg);
                            continue;
                        }
                    }
                }

                // Check for multiline input
                if trimmed.is_empty() && in_multiline {
                    // Empty line in multiline mode - execute the buffer
                    execute_code(engine, &multiline_buffer, line_number);
                    multiline_buffer.clear();
                    in_multiline = false;
                    line_number += 1;
                    continue;
                }

                if in_multiline {
                    multiline_buffer.push_str(&line);
                    multiline_buffer.push('\n');
                } else {
                    // Check if this line needs continuation
                    if needs_continuation(&line) {
                        multiline_buffer.push_str(&line);
                        multiline_buffer.push('\n');
                        in_multiline = true;
                    } else {
                        // Single line execution
                        execute_code(engine, &line, line_number);
                        line_number += 1;
                    }
                }

                rl.add_history_entry(&line)?;
            }
            
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                if in_multiline {
                    multiline_buffer.clear();
                    in_multiline = false;
                }
            }
            
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            
            Err(err) => {
                error!("REPL error: {}", err);
                break;
            }
        }
    }

    Ok(())
}

fn execute_code(engine: &mut BebionEngine, code: &str, line_number: usize) {
    if code.trim().is_empty() {
        return;
    }

    debug!("Executing code at line {}: {}", line_number, code);

    match engine.execute_script(code) {
        Ok(result) => {
            // TODO: Convert GcHandle to displayable value
            println!("{}", format!("=> [object]").bright_cyan());
        }
        Err(err) => {
            print_error(&err, line_number);
        }
    }
}

fn print_error(error: &BebionError, line_number: usize) {
    match error {
        BebionError::ParseError(msg) => {
            println!("{}: {} (line {})", "SyntaxError".red().bold(), msg, line_number);
        }
        BebionError::CompileError(msg) => {
            println!("{}: {} (line {})", "CompileError".red().bold(), msg, line_number);
        }
        BebionError::RuntimeError(msg) => {
            println!("{}: {} (line {})", "RuntimeError".red().bold(), msg, line_number);
        }
        BebionError::ModuleError(msg) => {
            println!("{}: {} (line {})", "ModuleError".red().bold(), msg, line_number);
        }
    }
}

enum ReplCommand {
    Exit,
    Continue,
    Error(String),
}

fn handle_repl_command(command: &str, engine: &mut BebionEngine) -> ReplCommand {
    match command {
        ".exit" | ".quit" => ReplCommand::Exit,
        
        ".help" => {
            show_help();
            ReplCommand::Continue
        }
        
        ".clear" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap_or(());
            ReplCommand::Continue
        }
        
        ".gc" => {
            let collected = engine.gc_collect();
            println!("Garbage collected {} objects", collected);
            ReplCommand::Continue
        }
        
        ".stats" => {
            show_stats(engine);
            ReplCommand::Continue
        }
        
        ".version" => {
            println!("Bebion v{}", env!("CARGO_PKG_VERSION"));
            ReplCommand::Continue
        }
        
        cmd if cmd.starts_with(".load ") => {
            let filename = &cmd[6..].trim();
            match std::fs::read_to_string(filename) {
                Ok(content) => {
                    execute_code(engine, &content, 0);
                    ReplCommand::Continue
                }
                Err(err) => ReplCommand::Error(format!("Failed to load {}: {}", filename, err)),
            }
        }
        
        cmd if cmd.starts_with(".save ") => {
            let filename = &cmd[6..].trim();
            ReplCommand::Error(format!("Save functionality not implemented: {}", filename))
        }
        
        _ => ReplCommand::Error(format!("Unknown command: {}", command)),
    }
}

fn show_help() {
    println!("{}", "REPL Commands:".bright_blue().bold());
    println!("  {}  - Show this help", ".help".yellow());
    println!("  {}  - Exit the REPL", ".exit".yellow());
    println!("  {}  - Clear the screen", ".clear".yellow());
    println!("  {}    - Force garbage collection", ".gc".yellow());
    println!("  {}  - Show runtime statistics", ".stats".yellow());
    println!("  {} - Show version information", ".version".yellow());
    println!("  {} - Load and execute a file", ".load <file>".yellow());
    println!("  {} - Save session to file", ".save <file>".yellow());
    println!();
    println!("{}", "JavaScript Features:".bright_blue().bold());
    println!("  • ECMAScript 2024 syntax");
    println!("  • async/await and Promises");
    println!("  • Classes and modules");
    println!("  • Template literals");
    println!("  • Destructuring");
    println!("  • Arrow functions");
    println!();
}

fn show_stats(engine: &BebionEngine) {
    println!("{}", "Runtime Statistics:".bright_blue().bold());
    
    let stats = engine.gc_stats();
    println!("Garbage Collector:");
    println!("  Total objects: {}", stats.total_objects);
    println!("  Young generation: {}", stats.young_objects);
    println!("  Old generation: {}", stats.old_objects);
    println!("  Root objects: {}", stats.root_objects);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Total collections: {}", stats.total_collections);
    println!("  Memory allocated: {} bytes", stats.bytes_allocated);
    println!("  Memory freed: {} bytes", stats.bytes_freed);
    
    let efficiency = if stats.total_allocations > 0 {
        (stats.bytes_freed as f64 / stats.bytes_allocated as f64) * 100.0
    } else {
        0.0
    };
    println!("  Collection efficiency: {:.1}%", efficiency);
}

fn needs_continuation(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Check for obvious continuation patterns
    if trimmed.ends_with('{') || 
       trimmed.ends_with('(') || 
       trimmed.ends_with('[') ||
       trimmed.ends_with(',') ||
       trimmed.ends_with('\\') {
        return true;
    }
    
    // Check for incomplete statements
    if trimmed.starts_with("function ") ||
       trimmed.starts_with("class ") ||
       trimmed.starts_with("if ") ||
       trimmed.starts_with("for ") ||
       trimmed.starts_with("while ") ||
       trimmed.starts_with("try ") ||
       trimmed.starts_with("switch ") {
        return true;
    }
    
    // Check for unmatched brackets
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escaped = false;
    
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        
        if ch == '\\' {
            escaped = true;
            continue;
        }
        
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        
        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }
    }
    
    paren_count > 0 || brace_count > 0 || bracket_count > 0 || in_string
}