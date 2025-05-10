use crate::error::{DocGenError, DocGenResult};
use crate::parser::{CodeItem, ParsedCode};
use crate::docstring::UpdatedDocstring;
use super::LanguageParser;
use tree_sitter::{Parser, Language, Query, QueryCursor};
use std::ops::Range;

extern "C" {
    fn tree_sitter_javascript() -> Language;
}

/// JavaScript language parser implementation
pub struct JavaScriptParser {
    parser: Parser,
}

impl JavaScriptParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_javascript() };
        parser.set_language(language).expect("Failed to load JavaScript grammar");
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
    
    /// Check if a given node is a JSDoc comment
    fn is_jsdoc_comment(&self, node_text: &str) -> bool {
        node_text.trim().starts_with("/**") && node_text.trim().ends_with("*/")
    }
    
    /// Extract parameters from a function declaration
    fn extract_parameters(&self, param_nodes: &[tree_sitter::Node], source: &str) -> Vec<String> {
        let mut params = Vec::new();
        
        for param_node in param_nodes {
            if param_node.kind() == "identifier" {
                let param_name = self.get_node_text(source, param_node.byte_range());
                params.push(param_name.to_string());
            } else if param_node.kind() == "assignment_pattern" {
                // Handle default parameters (e.g., x = 1)
                if let Some(left_node) = param_node.child(0) {
                    if left_node.kind() == "identifier" {
                        let param_name = self.get_node_text(source, left_node.byte_range());
                        params.push(format!("{}=", param_name));
                    }
                }
            } else if param_node.kind() == "rest_parameter" {
                // Handle rest parameters (e.g., ...args)
                if let Some(rest_node) = param_node.child_by_field_name("parameter") {
                    if rest_node.kind() == "identifier" {
                        let param_name = self.get_node_text(source, rest_node.byte_range());
                        params.push(format!("...{}", param_name));
                    }
                }
            } else if param_node.kind() == "object_pattern" || param_node.kind() == "array_pattern" {
                // Handle destructuring (simplified)
                let param_text = self.get_node_text(source, param_node.byte_range());
                params.push(param_text.to_string());
            }
        }
        
        params
    }
    
    /// Extract JSDoc comment and check if it's outdated
    fn extract_jsdoc(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        let mut comment_node = None;
        
        // Check for comments directly before the function declaration
        if cursor.goto_first_child() {
            if cursor.node().kind() == "comment" && self.is_jsdoc_comment(self.get_node_text(source, cursor.node().byte_range())) {
                comment_node = Some(cursor.node());
            }
        }
        
        // There's no goto_previous_sibling in tree-sitter, so we need to use a different approach
        // Look for comments in the source code before the node's start position
        if comment_node.is_none() {
            let node_start_position = node.start_position(); // We use this variable later in the function
            let node_start_byte = node.start_byte();
            
            // Use a substring of the source code up to the node start position
            let preceding_text = &source[..node_start_byte];
            
            // Look for the closest JSDoc comment
            if let Some(last_jsdoc_start) = preceding_text.rfind("/**") {
                if let Some(last_jsdoc_end) = preceding_text[last_jsdoc_start..].find("*/") {
                    let full_comment = &preceding_text[last_jsdoc_start..(last_jsdoc_start + last_jsdoc_end + 2)];
                    
                    // Check if it's close enough to be related to this node
                    let comment_lines_count = full_comment.matches('\n').count();
                    let comment_end_pos = preceding_text[..last_jsdoc_start].matches('\n').count() + comment_lines_count;
                    
                    // Check if the comment is immediately before the node (accounting for blank lines)
                    if node_start_position.row as usize - comment_end_pos <= 2 {
                        return Some(
                            full_comment.trim()
                                .trim_start_matches("/**")
                                .trim_end_matches("*/")
                                .lines()
                                .map(|line| line.trim().trim_start_matches("*").trim())
                                .collect::<Vec<_>>()
                                .join("\n")
                                .trim()
                                .to_string()
                        );
                    }
                }
            }
            
            return None;
        }
        
        // Extract the comment text if found
        comment_node.map(|node| {
            let comment_text = self.get_node_text(source, node.byte_range());
            
            // Clean up the comment (remove the /** and */ and trim)
            comment_text.trim()
                .trim_start_matches("/**")
                .trim_end_matches("*/")
                .lines()
                .map(|line| line.trim().trim_start_matches("*").trim())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string()
        })
    }
}

impl LanguageParser for JavaScriptParser {
    fn parse(&self, content: &str) -> DocGenResult<ParsedCode> {
        let mut code_items = Vec::new();
        
        // Parse the JavaScript code using tree-sitter
        // Since Parser doesn't implement Clone, we create a new one each time
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_javascript() };
        parser.set_language(language).expect("Failed to load JavaScript grammar");
        
        let tree = parser.parse(content, None)
            .ok_or_else(|| DocGenError::ParsingError("Failed to parse JavaScript code".into()))?;
        
        let root_node = tree.root_node();
        // We don't need cursor here, removing it
        
        // Query to find function and class declarations
        let function_query = Query::new(
            unsafe { tree_sitter_javascript() },
            "(function_declaration name: (identifier) @function_name) @function"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create function query: {}", e)))?;
        
        let method_query = Query::new(
            unsafe { tree_sitter_javascript() },
            "(method_definition name: (property_identifier) @method_name) @method"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create method query: {}", e)))?;
        
        let class_query = Query::new(
            unsafe { tree_sitter_javascript() },
            "(class_declaration name: (identifier) @class_name) @class"
        ).map_err(|e| DocGenError::ParsingError(format!("Failed to create class query: {}", e)))?;
        
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
                            let mut param_nodes = Vec::new();
                            let mut param_cursor = params_node.walk();
                            
                            if param_cursor.goto_first_child() {
                                while param_cursor.node().kind() != ")" {
                                    if param_cursor.node().kind() != "(" && param_cursor.node().kind() != "," {
                                        param_nodes.push(param_cursor.node());
                                    }
                                    if !param_cursor.goto_next_sibling() {
                                        break;
                                    }
                                }
                            }
                            
                            self.extract_parameters(&param_nodes, content)
                        } else {
                            Vec::new()
                        };
                        
                        // Extract JSDoc comment
                        let docstring = self.extract_jsdoc(function_node, content);
                        
                        code_items.push(CodeItem {
                            item_type: "function".to_string(),
                            name: function_name,
                            line_number,
                            code: self.extract_code_block(content, line_number, end_line),
                            existing_docstring: docstring,
                            parent: None,
                            parameters: params,
                            returns: None,
                            indentation: self.extract_indentation(content, line_number),
                        });
                    }
                }
            }
        }
        
        // Process class declarations
        query_cursor = QueryCursor::new();
        let class_matches = query_cursor.matches(&class_query, root_node, content.as_bytes());
        
        for class_match in class_matches {
            for capture in class_match.captures {
                if capture.index == 0 { // @class capture
                    let class_node = capture.node;
                    let name_node = &class_query.capture_names()[1]; // @class_name - borrow instead of moving
                    
                    if let Some(name_capture) = class_match.captures.iter().find(|c| &class_query.capture_names()[c.index as usize] == name_node) {
                        let class_name = self.get_node_text(content, name_capture.node.byte_range()).to_string();
                        let start_position = class_node.start_position();
                        let end_position = class_node.end_position();
                        let line_number = start_position.row + 1; // 1-indexed
                        let end_line = end_position.row + 1;
                        
                        // Extract JSDoc comment
                        let docstring = self.extract_jsdoc(class_node, content);
                        
                        code_items.push(CodeItem {
                            item_type: "class".to_string(),
                            name: class_name.clone(),
                            line_number,
                            code: self.extract_code_block(content, line_number, end_line),
                            existing_docstring: docstring,
                            parent: None,
                            parameters: Vec::new(),
                            returns: None,
                            indentation: self.extract_indentation(content, line_number),
                        });
                        
                        // Now process methods within the class
                        if let Some(class_body) = class_node.child_by_field_name("body") {
                            let mut method_query_cursor = QueryCursor::new();
                            let method_matches = method_query_cursor.matches(&method_query, class_body, content.as_bytes());
                            
                            for method_match in method_matches {
                                for method_capture in method_match.captures {
                                    if method_capture.index == 0 { // @method capture
                                        let method_node = method_capture.node;
                                        let method_name_node = &method_query.capture_names()[1]; // @method_name - borrow instead of moving
                                        
                                        if let Some(method_name_capture) = method_match.captures.iter().find(|c| &method_query.capture_names()[c.index as usize] == method_name_node) {
                                            let method_name = self.get_node_text(content, method_name_capture.node.byte_range()).to_string();
                                            let method_start = method_node.start_position();
                                            let method_end = method_node.end_position();
                                            let method_line = method_start.row + 1;
                                            let method_end_line = method_end.row + 1;
                                            
                                            // Find parameters
                                            let params = if let Some(params_node) = method_node.child_by_field_name("parameters") {
                                                let mut param_nodes = Vec::new();
                                                let mut param_cursor = params_node.walk();
                                                
                                                if param_cursor.goto_first_child() {
                                                    while param_cursor.node().kind() != ")" {
                                                        if param_cursor.node().kind() != "(" && param_cursor.node().kind() != "," {
                                                            param_nodes.push(param_cursor.node());
                                                        }
                                                        if !param_cursor.goto_next_sibling() {
                                                            break;
                                                        }
                                                    }
                                                }
                                                
                                                self.extract_parameters(&param_nodes, content)
                                            } else {
                                                Vec::new()
                                            };
                                            
                                            // Extract JSDoc comment
                                            let docstring = self.extract_jsdoc(method_node, content);
                                            
                                            code_items.push(CodeItem {
                                                item_type: "method".to_string(),
                                                name: method_name,
                                                line_number: method_line,
                                                code: self.extract_code_block(content, method_line, method_end_line),
                                                existing_docstring: docstring,
                                                parent: Some(class_name.clone()),
                                                parameters: params,
                                                returns: None,
                                                indentation: self.extract_indentation(content, method_line),
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
            
            // Get the line that defines the function/class/method
            let line_index = item.line_number - 1; // Convert to 0-based index
            
            if line_index >= lines.len() {
                return Err(DocGenError::UpdateError(
                    format!("Line number {} is out of bounds", item.line_number)));
            }
            
            // Unused variable - just remove the prefix and add underscore to avoid warning
            let _def_line = lines[line_index];
            
            // Get indentation level from the definition line
            let indentation = item.indentation.clone();
            
            // Check if there's an existing docstring to replace
            let mut has_existing_docstring = false;
            let mut docstring_start_line = line_index;
            let mut docstring_end_line = line_index;
            
            // Look for existing JSDoc comment
            for i in (0..line_index).rev() {
                let line = lines[i].trim();
                if line.starts_with("/**") {
                    has_existing_docstring = true;
                    docstring_start_line = i;
                    
                    // Find the end of the JSDoc comment
                    for j in i..line_index {
                        if lines[j].trim().contains("*/") {
                            docstring_end_line = j;
                            break;
                        }
                    }
                    break;
                } else if !line.is_empty() && !line.starts_with("//") {
                    // We found a non-comment, non-empty line, so there's no preceding docstring
                    break;
                }
            }
            
            // Format the JSDoc comment
            let mut jsdoc_lines = Vec::new();
            jsdoc_lines.push(format!("{}/**", indentation));
            
            // Add docstring lines with proper indentation
            for line in update.new_docstring.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    jsdoc_lines.push(format!("{} * {}", indentation, trimmed));
                } else {
                    jsdoc_lines.push(format!("{} *", indentation));
                }
            }
            
            jsdoc_lines.push(format!("{} */", indentation));
            let formatted_jsdoc = jsdoc_lines.join("\n");
            
            // Update the content
            if has_existing_docstring {
                // Replace existing JSDoc comment
                let before = if docstring_start_line > 0 {
                    lines[..docstring_start_line].join("\n")
                } else {
                    String::new()
                };
                
                let after = if docstring_end_line + 1 < lines.len() {
                    format!("\n{}", lines[(docstring_end_line + 1)..].join("\n"))
                } else {
                    String::new()
                };
                
                new_content = format!("{}\n{}{}", before, formatted_jsdoc, after);
            } else {
                // Insert new JSDoc comment before the definition
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
                
                new_content = format!("{}\n{}{}", before, formatted_jsdoc, after);
            }
        }
        
        Ok(new_content)
    }
}