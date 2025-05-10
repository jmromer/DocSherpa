use crate::error::{DocGenError, DocGenResult};
use crate::parser::{CodeItem, ParsedCode};
use crate::docstring::UpdatedDocstring;
use super::LanguageParser;
use tree_sitter::{Parser, Language, Query, QueryCursor};
use std::ops::Range;

extern "C" {
    fn tree_sitter_rust() -> Language;
}

/// Rust language parser implementation
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_rust() };
        parser.set_language(language).expect("Failed to load Rust grammar");
        Self { parser }
    }
    
    /// Extract a substring from the source based on a byte range
    fn get_node_text<'a>(&self, source: &'a str, range: Range<usize>) -> &'a str {
        &source[range.start..range.end]
    }
    
    /// Get the line number for a given byte position
    fn get_line_number(&self, source: &str, position: usize) -> usize {
        let mut line_number = 1;
        for (i, c) in source.char_indices() {
            if i >= position {
                break;
            }
            if c == '\n' {
                line_number += 1;
            }
        }
        line_number
    }
    
    /// Extract indentation from a line
    fn extract_indentation(&self, content: &str, line_number: usize) -> String {
        if let Some(line) = content.lines().nth(line_number - 1) {
            line.chars().take_while(|c| c.is_whitespace()).collect()
        } else {
            "".to_string()
        }
    }
    
    /// Extract a code block from the source content
    fn extract_code_block(&self, content: &str, start_line: usize, end_line: usize) -> String {
        content.lines()
            .skip(start_line - 1)
            .take(end_line - start_line + 1)
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Check if a given node is a documentation comment
    fn is_doc_comment(&self, node_kind: &str) -> bool {
        node_kind == "line_comment" && node_kind.starts_with("///")
    }
    
    /// Extract parameters from a function declaration
    fn extract_parameters(&self, params_node: tree_sitter::Node, source: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut cursor = params_node.walk();
        
        // Navigate to the parameters
        if cursor.goto_first_child() {
            // Skip the opening parenthesis
            if cursor.goto_next_sibling() {
                while cursor.node().kind() != ")" {
                    if cursor.node().kind() == "parameter" {
                        // Extract the parameter name
                        let mut param_cursor = cursor.node().walk();
                        if param_cursor.goto_first_child() {
                            // Find the pattern node (which contains the parameter name)
                            if param_cursor.node().kind() == "pattern" {
                                // Get the identifier inside the pattern
                                let mut pattern_cursor = param_cursor.node().walk();
                                if pattern_cursor.goto_first_child() && pattern_cursor.node().kind() == "identifier" {
                                    let param_name = self.get_node_text(source, pattern_cursor.node().byte_range());
                                    params.push(param_name.to_string());
                                }
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        
        params
    }
    
    /// Extract return type from a function declaration
    fn extract_return_type(&self, return_type_node: Option<tree_sitter::Node>, source: &str) -> Option<String> {
        return_type_node.map(|node| self.get_node_text(source, node.byte_range()).to_string())
    }
    
    /// Extract Rust doc comment blocks
    fn extract_doc_comment(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let mut comments = Vec::new();
        
        // Since tree-sitter doesn't have goto_previous_sibling, we need to use a different approach
        // Look for doc comments in the source code before the node's start position
        let _node_start_position = node.start_position(); // Prefix with underscore to indicate unused
        let node_start_byte = node.start_byte();
        
        // Use a substring of the source code up to the node start position
        let preceding_text = &source[..node_start_byte];
        
        // Count the newlines to determine line number
        let node_line_number = preceding_text.matches('\n').count() + 1;
        
        // Look back through the source code to find doc comments
        if node_line_number > 1 {
            let lines: Vec<&str> = source.lines().collect();
            let mut current_line = node_line_number - 1;
            let mut collecting_comments = false;
            
            while current_line > 0 {
                let line = lines[current_line - 1].trim();
                
                if line.starts_with("///") {
                    collecting_comments = true;
                    // Strip the /// and any leading space
                    let doc_text = line.trim_start_matches("///").trim();
                    comments.push(doc_text.to_string());
                } else if line.is_empty() {
                    // Skip empty lines
                } else if collecting_comments {
                    // Found a non-comment, non-empty line after we started collecting comments
                    break;
                } else {
                    // Found a non-comment, non-empty line before we started collecting comments
                    break;
                }
                
                current_line -= 1;
            }
        }
        
        if comments.is_empty() {
            None
        } else {
            // Reverse comments to get them in the right order
            comments.reverse();
            Some(comments.join("\n"))
        }
    }
}

impl LanguageParser for RustParser {
    fn parse(&self, content: &str) -> DocGenResult<ParsedCode> {
        let mut code_items = Vec::new();
        
        // Parse the Rust code using tree-sitter
        // Since Parser doesn't implement Clone, we create a new one each time
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_rust() };
        parser.set_language(language).expect("Failed to load Rust grammar");
        
        let tree = parser.parse(content, None)
            .ok_or_else(|| DocGenError::ParsingError("Failed to parse Rust code".into()))?;
        
        let root_node = tree.root_node();
        
        // Query to find function, struct, and impl declarations
        let function_query = Query::new(
            unsafe { tree_sitter_rust() },
            "(function_item name: (identifier) @function_name) @function"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create function query: {}", e)))?;
        
        let struct_query = Query::new(
            unsafe { tree_sitter_rust() },
            "(struct_item name: (type_identifier) @struct_name) @struct"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create struct query: {}", e)))?;
        
        let impl_query = Query::new(
            unsafe { tree_sitter_rust() },
            "(impl_item type: (type_identifier) @impl_type) @impl"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create impl query: {}", e)))?;
        
        let method_query = Query::new(
            unsafe { tree_sitter_rust() },
            "(impl_item (block (function_item name: (identifier) @method_name) @method))"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create method query: {}", e)))?;
        
        // Process function declarations
        let mut query_cursor = QueryCursor::new();
        let function_matches = query_cursor.matches(&function_query, root_node, content.as_bytes());
        
        for function_match in function_matches {
            for capture in function_match.captures {
                if capture.index == 0 { // @function capture
                    let function_node = capture.node;
                    let name_node = &function_query.capture_names()[1]; // @function_name - borrow instead of moving
                    
                    if let Some(name_capture) = function_match.captures.iter().find(|c| &function_query.capture_names()[c.index as usize] == name_node) {
                        let function_name = self.get_node_text(content, name_capture.node.byte_range()).to_string();
                        let start_position = function_node.start_position();
                        let end_position = function_node.end_position();
                        let line_number = start_position.row + 1; // 1-indexed
                        let end_line = end_position.row + 1;
                        
                        // Find parameters
                        let params = if let Some(params_node) = function_node.child_by_field_name("parameters") {
                            self.extract_parameters(params_node, content)
                        } else {
                            Vec::new()
                        };
                        
                        // Find return type
                        let return_type = self.extract_return_type(function_node.child_by_field_name("return_type"), content);
                        
                        // Extract doc comment
                        let docstring = self.extract_doc_comment(function_node, content);
                        
                        code_items.push(CodeItem {
                            item_type: "function".to_string(),
                            name: function_name,
                            line_number,
                            code: self.extract_code_block(content, line_number, end_line),
                            existing_docstring: docstring,
                            parent: None,
                            parameters: params,
                            returns: return_type,
                            indentation: self.extract_indentation(content, line_number),
                        });
                    }
                }
            }
        }
        
        // Process struct declarations
        query_cursor = QueryCursor::new();
        let struct_matches = query_cursor.matches(&struct_query, root_node, content.as_bytes());
        
        for struct_match in struct_matches {
            for capture in struct_match.captures {
                if capture.index == 0 { // @struct capture
                    let struct_node = capture.node;
                    let name_node = &struct_query.capture_names()[1]; // @struct_name - borrow instead of moving
                    
                    if let Some(name_capture) = struct_match.captures.iter().find(|c| &struct_query.capture_names()[c.index as usize] == name_node) {
                        let struct_name = self.get_node_text(content, name_capture.node.byte_range()).to_string();
                        let start_position = struct_node.start_position();
                        let end_position = struct_node.end_position();
                        let line_number = start_position.row + 1; // 1-indexed
                        let end_line = end_position.row + 1;
                        
                        // Extract doc comment
                        let docstring = self.extract_doc_comment(struct_node, content);
                        
                        code_items.push(CodeItem {
                            item_type: "struct".to_string(),
                            name: struct_name,
                            line_number,
                            code: self.extract_code_block(content, line_number, end_line),
                            existing_docstring: docstring,
                            parent: None,
                            parameters: Vec::new(),
                            returns: None,
                            indentation: self.extract_indentation(content, line_number),
                        });
                    }
                }
            }
        }
        
        // Process impl blocks and their methods
        query_cursor = QueryCursor::new();
        let impl_matches = query_cursor.matches(&impl_query, root_node, content.as_bytes());
        
        for impl_match in impl_matches {
            for capture in impl_match.captures {
                if capture.index == 0 { // @impl capture
                    let impl_node = capture.node;
                    let type_node = &impl_query.capture_names()[1]; // @impl_type - borrow instead of moving
                    
                    if let Some(type_capture) = impl_match.captures.iter().find(|c| &impl_query.capture_names()[c.index as usize] == type_node) {
                        let type_name = self.get_node_text(content, type_capture.node.byte_range()).to_string();
                        
                        // Process methods within the impl block
                        let mut method_cursor = QueryCursor::new();
                        
                        if let Some(block_node) = impl_node.child_by_field_name("body") {
                            let method_matches = method_cursor.matches(&method_query, block_node, content.as_bytes());
                            
                            for method_match in method_matches {
                                for method_capture in method_match.captures {
                                    if method_query.capture_names()[method_capture.index as usize] == "method" {
                                        let method_node = method_capture.node;
                                        
                                        // Find the method name
                                        if let Some(name_capture) = method_match.captures.iter()
                                            .find(|c| method_query.capture_names()[c.index as usize] == "method_name") {
                                            let method_name = self.get_node_text(content, name_capture.node.byte_range()).to_string();
                                            let start_position = method_node.start_position();
                                            let end_position = method_node.end_position();
                                            let line_number = start_position.row + 1; // 1-indexed
                                            let end_line = end_position.row + 1;
                                            
                                            // Find parameters
                                            let params = if let Some(params_node) = method_node.child_by_field_name("parameters") {
                                                self.extract_parameters(params_node, content)
                                            } else {
                                                Vec::new()
                                            };
                                            
                                            // Find return type
                                            let return_type = self.extract_return_type(method_node.child_by_field_name("return_type"), content);
                                            
                                            // Extract doc comment
                                            let docstring = self.extract_doc_comment(method_node, content);
                                            
                                            code_items.push(CodeItem {
                                                item_type: "method".to_string(),
                                                name: method_name,
                                                line_number,
                                                code: self.extract_code_block(content, line_number, end_line),
                                                existing_docstring: docstring,
                                                parent: Some(type_name.clone()),
                                                parameters: params,
                                                returns: return_type,
                                                indentation: self.extract_indentation(content, line_number),
                                            });
                                        }
                                    }
                                }
                            }
                        }
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
        let mut new_content = content.to_string();
        
        // Get access to the parsed code items for more accurate updates
        let parsed_code = self.parse(&new_content)?;
        
        // Sort updates in reverse order by line number to avoid line number shifts
        let mut sorted_updates = updated_docstrings.to_vec();
        sorted_updates.sort_by(|a, b| {
            let a_line = parsed_code.items[a.item_index].line_number;
            let b_line = parsed_code.items[b.item_index].line_number;
            b_line.cmp(&a_line)
        });
        
        for update in sorted_updates {
            let item = &parsed_code.items[update.item_index];
            let lines: Vec<&str> = new_content.lines().collect();
            
            // Get the line that defines the item
            let line_index = item.line_number - 1; // Convert to 0-based index
            
            if line_index >= lines.len() {
                return Err(DocGenError::UpdateError(
                    format!("Line number {} is out of bounds", item.line_number)));
            }
            
            // Get indentation level from the definition line
            let indentation = item.indentation.clone();
            
            // Check if there's an existing doc comment to replace
            let mut has_existing_docstring = false;
            let mut doc_start_line = line_index;
            let mut doc_end_line = line_index;
            
            // Look for existing doc comments
            for i in (0..line_index).rev() {
                let line = lines[i].trim();
                if line.starts_with("///") {
                    has_existing_docstring = true;
                    doc_start_line = i;
                } else if !line.is_empty() {
                    // We found a non-comment, non-empty line, so stop looking
                    break;
                }
            }
            
            if has_existing_docstring {
                // Find the end of the doc comment block
                for i in (doc_start_line..line_index).rev() {
                    if lines[i].trim().starts_with("///") {
                        doc_end_line = i;
                        break;
                    }
                }
            }
            
            // Format the new docstring as Rust doc comments
            let new_doc_lines: Vec<String> = update.new_docstring
                .lines()
                .map(|line| {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        format!("{}/// {}", indentation, trimmed)
                    } else {
                        format!("{}///", indentation)
                    }
                })
                .collect();
            
            let formatted_doc = new_doc_lines.join("\n");
            
            // Update the content
            if has_existing_docstring {
                // Replace existing doc comment
                let before = if doc_start_line > 0 {
                    lines[..doc_start_line].join("\n")
                } else {
                    String::new()
                };
                
                let after = if doc_end_line + 1 < lines.len() {
                    format!("\n{}", lines[(doc_end_line + 1)..].join("\n"))
                } else {
                    String::new()
                };
                
                new_content = format!("{}\n{}{}", before, formatted_doc, after);
            } else {
                // Insert new doc comment before the definition
                let before = if line_index > 0 {
                    lines[..line_index].join("\n")
                } else {
                    String::new()
                };
                
                let after = if line_index < lines.len() {
                    format!("\n{}", lines[line_index..].join("\n"))
                } else {
                    String::new()
                };
                
                new_content = format!("{}\n{}{}", before, formatted_doc, after);
            }
        }
        
        Ok(new_content)
    }
}