use rustpython_parser::{parser, ast::{self, Mod}};
use crate::error::{DocGenError, DocGenResult};
use crate::parser::{CodeItem, ParsedCode};
use crate::docstring::UpdatedDocstring;
use super::LanguageParser;

/// Python language parser implementation
pub struct PythonParser;

impl PythonParser {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract docstring from an AST node
    fn extract_docstring(&self, body: &[ast::Located<ast::StmtKind>]) -> Option<String> {
        if let Some(stmt) = body.first() {
            if let ast::StmtKind::Expr { value } = &stmt.node {
                if let ast::ExprKind::Constant { value: ast::Constant::Str(s), .. } = &value.node {
                    return Some(s.to_string());
                }
            }
        }
        None
    }
    
    /// Extract parameters from a function definition
    fn extract_parameters(&self, args: &ast::Arguments) -> Vec<String> {
        let mut params = Vec::new();
        
        // Add self parameter for methods if present
        for arg in &args.posonlyargs {
            let ast::ArgData { arg, .. } = &arg.node;
            if arg == "self" {
                params.push("self".to_string());
                break;
            }
        }
        
        // Extract positional arguments
        for arg in &args.args {
            let ast::ArgData { arg, .. } = &arg.node;
            if arg != "self" {
                params.push(arg.clone());
            }
        }
        
        // Extract positional-only arguments (other than self)
        for arg in &args.posonlyargs {
            let ast::ArgData { arg, .. } = &arg.node;
            if arg != "self" && !params.contains(arg) {
                params.push(arg.clone());
            }
        }
        
        // Extract keyword arguments
        for arg in &args.kwonlyargs {
            let ast::ArgData { arg, .. } = &arg.node;
            params.push(format!("{}=", arg));
        }
        
        // Add *args if present
        if let Some(vararg) = &args.vararg {
            let ast::ArgData { arg, .. } = &vararg.node;
            params.push(format!("*{}", arg));
        }
        
        // Add **kwargs if present
        if let Some(kwarg) = &args.kwarg {
            let ast::ArgData { arg, .. } = &kwarg.node;
            params.push(format!("**{}", arg));
        }
        
        params
    }
    
    /// Extract return type annotation if available
    fn extract_return_type(&self, returns: &Option<Box<ast::Located<ast::ExprKind>>>) -> Option<String> {
        returns.as_ref().map(|expr| format!("{:?}", expr))
    }
    
    /// Extract a code block from the source content
    fn extract_code_block(&self, content: &str, start_line: usize, end_line: usize) -> String {
        content.lines()
            .skip(start_line - 1)
            .take(end_line - start_line + 1)
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Extract indentation from a line
    fn extract_indentation(&self, content: &str, line_number: usize) -> String {
        if let Some(line) = content.lines().nth(line_number - 1) {
            line.chars().take_while(|c| c.is_whitespace()).collect()
        } else {
            "".to_string()
        }
    }
}

impl LanguageParser for PythonParser {
    fn parse(&self, content: &str) -> DocGenResult<ParsedCode> {
        // Parse Python code using rustpython-parser
        let statements = parser::parse_program(content, "<string>")
            .map_err(|e| DocGenError::ParsingError(format!("Failed to parse Python code: {}", e)))?;
        
        // Process each statement in the module
        let mut code_items = Vec::new();
        
        for stmt in &statements {
            match &stmt.node {
                ast::StmtKind::FunctionDef { name, args, body, decorator_list: _, returns, type_comment: _ } => {
                    let docstring = self.extract_docstring(body);
                    let lineno = stmt.location.row();
                    let end_lineno = stmt.end_location.map(|loc| loc.row()).unwrap_or(lineno);
                    
                    code_items.push(CodeItem {
                        item_type: "function".to_string(),
                        name: name.to_string(),
                        line_number: lineno,
                        code: self.extract_code_block(content, lineno, end_lineno),
                        existing_docstring: docstring,
                        parent: None,
                        parameters: self.extract_parameters(args),
                        returns: self.extract_return_type(returns),
                        indentation: self.extract_indentation(content, lineno),
                    });
                },
                ast::StmtKind::ClassDef { name, body, decorator_list: _, bases: _, keywords: _ } => {
                    let class_docstring = self.extract_docstring(body);
                    let class_lineno = stmt.location.row();
                    let class_end_lineno = stmt.end_location.map(|loc| loc.row()).unwrap_or(class_lineno);
                    
                    // Add the class itself
                    code_items.push(CodeItem {
                        item_type: "class".to_string(),
                        name: name.to_string(),
                        line_number: class_lineno,
                        code: self.extract_code_block(content, class_lineno, class_end_lineno),
                        existing_docstring: class_docstring,
                        parent: None,
                        parameters: Vec::new(),
                        returns: None,
                        indentation: self.extract_indentation(content, class_lineno),
                    });
                    
                    // Process class methods
                    for class_stmt in body {
                        if let ast::StmtKind::FunctionDef { name: method_name, args, body: method_body, decorator_list: _, returns, type_comment: _ } = &class_stmt.node {
                            let docstring = self.extract_docstring(method_body);
                            let method_lineno = class_stmt.location.row();
                            let method_end_lineno = class_stmt.end_location.map(|loc| loc.row()).unwrap_or(method_lineno);
                            
                            code_items.push(CodeItem {
                                item_type: "method".to_string(),
                                name: method_name.to_string(),
                                line_number: method_lineno,
                                code: self.extract_code_block(content, method_lineno, method_end_lineno),
                                existing_docstring: docstring,
                                parent: Some(name.to_string()),
                                parameters: self.extract_parameters(args),
                                returns: self.extract_return_type(returns),
                                indentation: self.extract_indentation(content, method_lineno),
                            });
                        }
                    }
                },
                _ => {} // Ignore other statement types
            }
        }
        
        // Return the parsed code
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
            
            let def_line = lines[line_index];
            
            // Get indentation level from the definition line
            let indentation = item.indentation.clone();
            
            // Check if there's an existing docstring to replace
            let mut has_existing_docstring = false;
            let mut docstring_end_line = line_index;
            
            // If the next line starts with triple quotes, we have a docstring to replace
            if line_index + 1 < lines.len() {
                let next_line = lines[line_index + 1].trim();
                if next_line.starts_with("\"\"\"") || next_line.starts_with("'''") {
                    has_existing_docstring = true;
                    
                    // Find the end of the docstring
                    for i in (line_index + 1)..lines.len() {
                        if i == line_index + 1 && lines[i].trim().ends_with("\"\"\"") && lines[i].trim().starts_with("\"\"\"") {
                            // Single line docstring
                            docstring_end_line = i;
                            break;
                        } else if i > line_index + 1 && (lines[i].trim().ends_with("\"\"\"") || lines[i].trim().ends_with("'''")) {
                            // Multi-line docstring
                            docstring_end_line = i;
                            break;
                        }
                    }
                }
            }
            
            // Add indentation to docstring lines
            let indented_docstring = update.new_docstring
                .lines()
                .enumerate()
                .map(|(_i, line)| {
                    // Indent all lines properly - add 4 spaces to align with Python code indentation
                    format!("{}    {}", indentation, line)
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            // Split content for insertion or replacement
            if has_existing_docstring {
                // Replace existing docstring
                let start_line = lines[..=line_index].join("\n");
                let end_line = if docstring_end_line + 1 < lines.len() {
                    lines[(docstring_end_line + 1)..].join("\n")
                } else {
                    "".to_string()
                };
                
                new_content = format!("{}\n{}\n{}", start_line, indented_docstring, end_line);
            } else {
                // Insert new docstring after definition line
                let split_index = if line_index > 0 {
                    // Add 1 for the newline character
                    lines[..line_index].join("\n").len() + def_line.len() + 1
                } else {
                    def_line.len()
                };
                
                let (before, after) = new_content.split_at(split_index);
                new_content = format!("{}\n{}{}", before, indented_docstring, after);
            }
        }
        
        Ok(new_content)
    }
}