mod config;
mod docstring;
mod error;
mod llm;
mod parser;
mod updater;
mod lang;

use crate::lang::LanguageParser;

use clap::{Parser, ArgAction, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;
use anyhow::Result;

/// Supported programming languages
#[derive(Debug, Clone, ValueEnum)]
enum Language {
    /// Python language support
    Python,
    /// Rust language support
    Rust,
    /// JavaScript language support
    JavaScript,
    /// TypeScript language support
    TypeScript,
    /// Automatically detect based on file extension
    Auto,
}

/// DocGen: A tool to generate or update documentation in code files using LLM
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Files to process
    #[clap(required = true)]
    files: Vec<PathBuf>,

    /// Programming language mode
    #[clap(short, long, value_enum, default_value = "auto")]
    language: Language,

    /// LLM provider to use (openai or claude)
    #[clap(short, long, default_value = "openai")]
    provider: String,

    /// Check mode - only report issues without making changes
    #[clap(short, long, action = ArgAction::SetTrue)]
    check: bool,

    /// Verbose mode - show more details
    #[clap(short, long, action = ArgAction::SetTrue)]
    verbose: bool,
    
    /// Test mode - analyze files without making API calls
    #[clap(long, action = ArgAction::SetTrue)]
    test: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create configuration
    let config = config::Config {
        provider: args.provider,
        check_only: args.check,
        verbose: args.verbose,
        test_mode: args.test,
    };
    
    if args.verbose {
        println!("{}", "DocGen: Documentation Generator".green().bold());
        println!("{} {:?}", "Processing files:".blue(), args.files);
    }
    
    // Process each file
    for file_path in &args.files {
        let language = match args.language {
            Language::Auto => detect_language(file_path),
            _ => args.language.clone(),
        };
        
        if config.verbose {
            println!("Detected language: {:?}", language);
        }
        
        process_file(file_path, &language, &config).await?;
    }
    
    Ok(())
}

/// Detect programming language from file extension
fn detect_language(file_path: &PathBuf) -> Language {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => Language::Python,
        Some("rs") => Language::Rust,
        Some("js") => Language::JavaScript,
        Some("ts") | Some("tsx") => Language::TypeScript,
        _ => {
            eprintln!("Warning: Could not detect language for {}. Defaulting to Python.", 
                     file_path.display());
            Language::Python
        }
    }
}

async fn process_file(file_path: &PathBuf, _language: &Language, config: &config::Config) -> Result<()> {
    if config.verbose {
        println!("\n{} {}", "Processing:".blue(), file_path.display());
    }
    
    // Read file content
    let content = std::fs::read_to_string(file_path)?;
    
    // Parse code based on language
    // For now, only Python is fully implemented
    let parser = lang::python::PythonParser::new();
    let parsed_code = parser.parse(&content)?;
    
    // Future implementation when tree-sitter is fixed:
    // let parsed_code = match language {
    //     Language::Python => {
    //         let parser = lang::python::PythonParser::new();
    //         parser.parse(&content)?
    //     },
    //     Language::Rust => {
    //         let parser = lang::rust::RustParser::new();
    //         parser.parse(&content)?
    //     },
    //     Language::JavaScript => {
    //         let parser = lang::javascript::JavaScriptParser::new();
    //         parser.parse(&content)?
    //     },
    //     Language::TypeScript => {
    //         let parser = lang::typescript::TypeScriptParser::new();
    //         parser.parse(&content)?
    //     },
    //     Language::Auto => unreachable!("Language should be resolved by this point"),
    // };
    
    // Analyze docstrings
    let docstring_issues = docstring::analyze(&parsed_code)?;
    
    if docstring_issues.is_empty() {
        if config.verbose {
            println!("{} {}", "✓".green(), "All items are properly documented".green());
        }
        return Ok(());
    }
    
    // Report issues
    println!("{} found {} documentation issues in {}", 
        "DocGen:".yellow(),
        docstring_issues.len(),
        file_path.display());
    
    for issue in &docstring_issues {
        println!("  {} {}: {}", "→".yellow(), issue.item_type, issue.name);
        if config.verbose {
            println!("    Line {}: {}", issue.line_number, issue.issue_type);
        }
    }
    
    // Exit if we're just checking or in test mode
    if config.check_only || config.test_mode {
        if config.test_mode && config.verbose {
            println!("{} Test mode - skipping LLM API calls", "DocGen:".blue());
            
            // Print parsed code items for verification
            println!("\n{} Parsed code items:", "DocGen:".blue());
            for (index, item) in parsed_code.items.iter().enumerate() {
                println!("  Item {}: {} '{}'", index, item.item_type, item.name);
                println!("    Line: {}", item.line_number);
                println!("    Parameters: {:?}", item.parameters);
                println!("    Docstring: {}", item.existing_docstring.as_ref().map_or("None", |s| s));
                println!();
            }
        }
        return Ok(());
    }
    
    // Use LLM to generate docstrings
    println!("{} Generating documentation using {}...", 
        "DocGen:".blue(),
        config.provider);
    
    let llm_client = llm::get_client(&config.provider)?;
    let updated_docstrings = llm_client.generate_docstrings(&parsed_code, &docstring_issues).await?;
    
    // Update the file with new docstrings
    // For now, only Python is fully implemented
    let updated_content = parser.update_content(&content, &updated_docstrings)?;
    
    // Future implementation when tree-sitter is fixed:
    // let updated_content = match language {
    //     Language::Python => {
    //         let parser = lang::python::PythonParser::new();
    //         parser.update_content(&content, &updated_docstrings)?
    //     },
    //     Language::Rust => {
    //         let parser = lang::rust::RustParser::new();
    //         parser.update_content(&content, &updated_docstrings)?
    //     },
    //     Language::JavaScript => {
    //         let parser = lang::javascript::JavaScriptParser::new();
    //         parser.update_content(&content, &updated_docstrings)?
    //     },
    //     Language::TypeScript => {
    //         let parser = lang::typescript::TypeScriptParser::new();
    //         parser.update_content(&content, &updated_docstrings)?
    //     },
    //     Language::Auto => unreachable!("Language should be resolved by this point"),
    // };
    
    // Write back to file
    std::fs::write(file_path, updated_content)?;
    
    println!("{} Updated documentation in {}", 
        "DocGen:".green(),
        file_path.display());
    
    Ok(())
}
