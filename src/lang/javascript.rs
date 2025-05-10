use crate::error::{DocGenError, DocGenResult};
use crate::parser::{CodeItem, ParsedCode};
use crate::docstring::UpdatedDocstring;
use super::LanguageParser;
use regex::RegexBuilder;

/// JavaScript language parser implementation using regex for simplicity
pub struct JavaScriptParser {}

impl JavaScriptParser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Extract indentation from a line
    fn extract_indentation(&self, line: &str) -> String {
        line.chars().take_while(|c| c.is_whitespace()).collect()
    }
    
    /// Extract parameters from a function declaration
    fn extract_parameters(&self, params_str: &str) -> Vec<String> {
        let cleaned = params_str.trim();
        
        if cleaned.is_empty() {
            return Vec::new();
        }
        
        cleaned
            .split(',')
            .map(|p| p.trim().to_string())
            .collect()
    }
    
    /// Check if a line is a JSDoc comment start
    fn is_jsdoc_start(&self, line: &str) -> bool {
        line.trim().starts_with("/**")
    }
    
    /// Check if a line is a JSDoc comment end
    fn is_jsdoc_end(&self, line: &str) -> bool {
        line.trim().ends_with("*/")
    }
    
    /// Extract JSDoc comment text
    fn extract_jsdoc(&self, content: &str, start_line: usize) -> Option<(String, usize)> {
        let lines: Vec<&str> = content.lines().collect();
        
        // Check if there's a JSDoc comment before the current position
        if start_line == 0 || !self.is_jsdoc_start(lines[start_line - 1]) {
            return None;
        }
        
        // Find JSDoc comment bounds
        let mut docstring_start = start_line - 1;
        while docstring_start > 0 && !self.is_jsdoc_start(lines[docstring_start]) {
            docstring_start -= 1;
        }
        
        let mut docstring_end = docstring_start;
        while docstring_end < start_line && !self.is_jsdoc_end(lines[docstring_end]) {
            docstring_end += 1;
        }
        
        if docstring_end == lines.len() || !self.is_jsdoc_end(lines[docstring_end]) {
            return None;
        }
        
        // Extract JSDoc content (clean up comment markers and indentation)
        let doc_lines: Vec<String> = lines[docstring_start..=docstring_end]
            .iter()
            .map(|line| {
                line.trim()
                    .trim_start_matches("/**")
                    .trim_start_matches("*")
                    .trim_start_matches(" ")
                    .trim_end_matches("*/")
                    .to_string()
            })
            .collect();
        
        Some((doc_lines.join("\n"), docstring_end - docstring_start + 1))
    }
}

impl LanguageParser for JavaScriptParser {
    fn parse(&self, content: &str) -> DocGenResult<ParsedCode> {
        let mut code_items = Vec::new();
        
        // Regex patterns for JavaScript
        let function_pattern = RegexBuilder::new(r"function\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\(([^)]*)\)")
            .multi_line(true)
            .build()
            .map_err(|e| DocGenError::ParsingError(format!("Failed to compile regex: {}", e)))?;
            
        let class_pattern = RegexBuilder::new(r"class\s+([a-zA-Z_$][a-zA-Z0-9_$]*)")
            .multi_line(true)
            .build()
            .map_err(|e| DocGenError::ParsingError(format!("Failed to compile regex: {}", e)))?;
            
        // More specific method pattern to avoid matching if/for statements
        // Methods are defined at the class level with proper indentation
        // We filter the keywords later in the code
        let method_pattern = RegexBuilder::new(r"(?:^|\n|\r)\s+(?:async\s+)?([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\(([^)]*)\)\s*(?:\{|$)")
            .multi_line(true)
            .build()
            .map_err(|e| DocGenError::ParsingError(format!("Failed to compile regex: {}", e)))?;
        
        // Track line numbers
        let lines: Vec<&str> = content.lines().collect();
        
        // Find function declarations
        for func_match in function_pattern.captures_iter(content) {
            let func_name = func_match.get(1).unwrap().as_str();
            let params_str = func_match.get(2).unwrap().as_str();
            
            // Find line number
            let func_start = func_match.get(0).unwrap().start();
            let prefix = &content[..func_start];
            let line_number = prefix.chars().filter(|&c| c == '\n').count();
            
            // Extract code block
            let line = lines[line_number];
            let indentation = self.extract_indentation(line);
            
            // Check for docstring
            let docstring = if line_number > 0 {
                self.extract_jsdoc(content, line_number)
                    .map(|(doc, _)| doc)
            } else {
                None
            };
            
            // Add to items
            code_items.push(CodeItem {
                item_type: "function".to_string(),
                name: func_name.to_string(),
                line_number: line_number + 1, // 1-indexed lines
                code: line.to_string(),
                existing_docstring: docstring,
                parent: None,
                parameters: self.extract_parameters(params_str),
                returns: None,
                indentation,
            });
        }
        
        // Track class start positions to avoid duplicates
        let mut processed_class_positions = std::collections::HashSet::new();
        
        // Find class declarations
        for class_match in class_pattern.captures_iter(content) {
            let class_name = class_match.get(1).unwrap().as_str();
            
            // Find line number
            let class_start = class_match.get(0).unwrap().start();
            
            // Skip if we've already seen a class at this position
            if processed_class_positions.contains(&class_start) {
                continue;
            }
            processed_class_positions.insert(class_start);
            
            let prefix = &content[..class_start];
            let line_number = prefix.chars().filter(|&c| c == '\n').count();
            
            // Extract code block
            let line = lines[line_number];
            let indentation = self.extract_indentation(line);
            
            // Check for docstring
            let docstring = if line_number > 0 {
                self.extract_jsdoc(content, line_number)
                    .map(|(doc, _)| doc)
            } else {
                None
            };
            
            // Get class definition scope - find matching closing brace
            let class_code = if line_number < lines.len() {
                let mut scope_level = 0;
                let mut end_line = line_number;
                let mut found_closing = false;
                
                for i in line_number..lines.len() {
                    let l = lines[i];
                    for c in l.chars() {
                        if c == '{' {
                            scope_level += 1;
                        } else if c == '}' {
                            scope_level -= 1;
                            if scope_level == 0 {
                                end_line = i;
                                found_closing = true;
                                break;
                            }
                        }
                    }
                    if found_closing {
                        break;
                    }
                }
                
                lines[line_number..=end_line].join("\n")
            } else {
                line.to_string()
            };
            
            // Add class to items
            code_items.push(CodeItem {
                item_type: "class".to_string(),
                name: class_name.to_string(),
                line_number: line_number + 1, // 1-indexed lines
                code: class_code.clone(), // Clone to avoid ownership issues
                existing_docstring: docstring.clone(),
                parent: None,
                parameters: Vec::new(),
                returns: None,
                indentation: indentation.clone(),
            });
            
            // Extract methods inside the class
            if let Some(class_block_start) = class_code.find('{') {
                let class_body = &class_code[class_block_start..];
                
                for method_match in method_pattern.captures_iter(class_body) {
                    let method_name = method_match.get(1).unwrap().as_str();
                    // Skip constructor and control structure keywords
                    if method_name == "constructor" || method_name == "super" || method_name == "return" ||
                       method_name == "if" || method_name == "for" || method_name == "while" || 
                       method_name == "switch" || method_name == "function" {
                        continue;
                    }
                    
                    let params_str = method_match.get(2).unwrap().as_str();
                    
                    // Find line number relative to class start
                    let method_start = method_match.get(0).unwrap().start();
                    let method_prefix = &class_body[..method_start];
                    let method_line_offset = method_prefix.chars().filter(|&c| c == '\n').count();
                    let method_line_number = line_number + method_line_offset;
                    
                    if method_line_number < lines.len() {
                        let method_line = lines[method_line_number];
                        let method_indentation = self.extract_indentation(method_line);
                        
                        // Check for docstring
                        let method_docstring = if method_line_number > 0 {
                            self.extract_jsdoc(content, method_line_number)
                                .map(|(doc, _)| doc)
                        } else {
                            None
                        };
                        
                        // Add method to items
                        code_items.push(CodeItem {
                            item_type: "method".to_string(),
                            name: method_name.to_string(),
                            line_number: method_line_number + 1, // 1-indexed lines
                            code: method_line.to_string(),
                            existing_docstring: method_docstring,
                            parent: Some(class_name.to_string()),
                            parameters: self.extract_parameters(params_str),
                            returns: None,
                            indentation: method_indentation,
                        });
                    }
                }
            }
        }
        
        Ok(ParsedCode {
            items: code_items,
            original_content: content.to_string(),
        })
    }
    
    fn update_content(&self, content: &str, updated_docstrings: &[UpdatedDocstring]) -> DocGenResult<String> {
        // Create a completely new approach using temporary files to avoid position issues
        use std::fs::File;
        use std::io::{Write, BufRead, BufReader};
        use tempfile::tempdir;
        
        let dir = tempdir().map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create temp dir: {}", e))))?;
        
        // Input path
        let input_path = dir.path().join("input.js");
        
        // Write original content to input file
        {
            let mut file = File::create(&input_path)
                .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create temp file: {}", e))))?;
            file.write_all(content.as_bytes())
                .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write to temp file: {}", e))))?;
        }
        
        // Output path
        let output_path = dir.path().join("output.js");
        let mut out_file = File::create(&output_path)
            .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create output file: {}", e))))?;
        
        // Get the items
        let parsed = self.parse(content)?;
        let items = parsed.items;
        
        // Create a map of line numbers to docstrings for quick lookup
        let mut docstrings_by_line = std::collections::HashMap::new();
        for docstring in updated_docstrings {
            if docstring.item_index < items.len() {
                let line_num = items[docstring.item_index].line_number;
                
                // Format docstring in JSDoc style
                let indentation = &docstring.indentation;
                let doc_lines: Vec<String> = docstring.new_docstring
                    .lines()
                    .map(|line| {
                        if line.trim().is_empty() {
                            format!("{}* ", indentation)
                        } else {
                            format!("{}* {}", indentation, line)
                        }
                    })
                    .collect();
                
                let formatted_docstring = format!("{}/**\n{}\n{}*/\n", 
                    indentation,
                    doc_lines.join("\n"),
                    indentation
                );
                
                docstrings_by_line.insert(line_num, formatted_docstring);
            }
        }
        
        // Now process the file line by line
        let file = File::open(&input_path)
            .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to open temp file: {}", e))))?;
        let reader = BufReader::new(file);
        
        let mut line_num = 1;
        let mut skipping_docstring = false;
        let mut inserted_docstrings = std::collections::HashSet::new();
        
        for line_result in reader.lines() {
            let line = line_result.map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read line: {}", e))))?;
            
            // If we're at the start of an existing docstring, skip it
            if self.is_jsdoc_start(&line) {
                skipping_docstring = true;
                continue;
            }
            
            // If we're at the end of an existing docstring, stop skipping
            if skipping_docstring && self.is_jsdoc_end(&line) {
                skipping_docstring = false;
                continue;
            }
            
            // Skip lines that are part of an existing docstring
            if skipping_docstring {
                continue;
            }
            
            // Check if we need to insert a docstring at this line
            if docstrings_by_line.contains_key(&line_num) && !inserted_docstrings.contains(&line_num) {
                // Insert the docstring
                let docstring = docstrings_by_line.get(&line_num).unwrap();
                writeln!(out_file, "{}", docstring.trim_end())
                    .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write docstring: {}", e))))?;
                inserted_docstrings.insert(line_num);
            }
            
            // Write the current line
            writeln!(out_file, "{}", line)
                .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write line: {}", e))))?;
            
            line_num += 1;
        }
        
        // Read final output
        let result = std::fs::read_to_string(&output_path)
            .map_err(|e| DocGenError::FileError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read output file: {}", e))))?;
        
        Ok(result)
    }
}