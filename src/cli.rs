use crate::lexer;
use crate::parser;
use crate::compiler;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const VERSION: &str = "1.0.3-alpha.1";

fn show_help() {
    println!("Quark Compiler v{}", VERSION);
    println!("Usage: quark <command> [options]");
    println!();
    println!("Commands:");
    println!("  build <file.qrk>    Compile a program");
    println!("  run <file.qrk>      Compile and run");
    println!("  check <file.qrk>    Check syntax");
    println!("  help                Show this help");
    println!("  version             Show version");
    println!();
    println!("Options for build/run:");
    println!("  -o, --output <name>  Output file name");
    println!();
    println!("Examples:");
    println!("  quark build hello.qrk");
    println!("  quark build hello.qrk -o program.exe");
    println!("  quark run hello.qrk");
    println!("  quark check test.qrk");
}

fn show_version() {
    println!("Quark Compiler v{}", VERSION);
    println!("License: Apache 2.0");
}

fn read_source_file(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    
    if path.extension().and_then(|s| s.to_str()) != Some("qrk") {
        return Err("File must have .qrk extension".to_string());
    }
    
    fs::read_to_string(path).map_err(|e| format!("Error reading file: {}", e))
}

fn compile_file(
    input_path: &str,
    output_path: Option<&str>,
) -> Result<PathBuf, String> {
    let start_time = Instant::now();
    
    let source = read_source_file(input_path)?;
    println!("Reading: {}", input_path);
    
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e.message))?;
    println!("Tokens: {}", tokens.len());
    
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    println!("Statements: {}", program.statements.len());
    
    let output_path = if let Some(path) = output_path {
        PathBuf::from(path)
    } else {
        let stem = Path::new(input_path)
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("output");
        
        let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        PathBuf::from(format!("{}{}", stem, ext))
    };
    
    let mut compiler = compiler::Compiler::new();
    compiler.compile_to_exe(&program, output_path.to_str().unwrap())
        .map_err(|e| format!("Compilation error: {:?}", e))?;
    
    let duration = start_time.elapsed();
    println!("Compilation time: {:.2?}", duration);
    
    Ok(output_path)
}

fn run_file(input_path: &str) -> Result<(), String> {
    let temp_dir = env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let exe_name = if cfg!(target_os = "windows") {
        format!("quark_temp_{}.exe", timestamp)
    } else {
        format!("quark_temp_{}", timestamp)
    };
    
    let exe_path = temp_dir.join(exe_name);
    
    let output = compile_file(input_path, exe_path.to_str())?;
    
    println!("Running program...");
    println!("------------------");
    
    let status = std::process::Command::new(&output)
        .status()
        .map_err(|e| format!("Execution error: {}", e))?;
    
    println!("------------------");
    
    let _ = std::fs::remove_file(&output);
    
    if !status.success() {
        return Err(format!("Program exited with code: {}", status));
    }
    
    Ok(())
}

fn check_syntax(input_path: &str) -> Result<(), String> {
    let source = read_source_file(input_path)?;
    
    println!("Checking syntax: {}", input_path);
    
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e.message))?;
    
    let mut parser = parser::Parser::new(tokens);
    parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    println!("Syntax is correct");
    Ok(())
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        show_help();
        return;
    }
    
    match args[1].as_str() {
        "help" | "--help" | "-h" => show_help(),
        "version" | "--version" | "-v" => show_version(),
        
        "build" => {
            if args.len() < 3 {
                eprintln!("Error: No file specified");
                eprintln!("Usage: quark build <file.qrk> [-o <output>]");
                return;
            }
            
            let mut input_file = &args[2];
            let mut output_file = None;
            let mut i = 3;
            
            while i < args.len() {
                match args[i].as_str() {
                    "-o" | "--output" => {
                        if i + 1 < args.len() {
                            output_file = Some(args[i + 1].as_str());
                            i += 2;
                        } else {
                            eprintln!("Error: {} requires a file name", args[i]);
                            return;
                        }
                    }
                    _ => {
                        input_file = &args[i];
                        i += 1;
                    }
                }
            }
            
            match compile_file(input_file, output_file) {
                Ok(output) => {
                    println!("Done. Executable: {}", output.display());
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: No file specified");
                eprintln!("Usage: quark run <file.qrk>");
                return;
            }
            
            if let Err(err) = run_file(&args[2]) {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
        
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: No file specified");
                eprintln!("Usage: quark check <file.qrk>");
                return;
            }
            
            if let Err(err) = check_syntax(&args[2]) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        
        cmd => {
            eprintln!("Unknown command: '{}'", cmd);
            eprintln!("Use 'quark help' for command list");
            std::process::exit(1);
        }
    }
}