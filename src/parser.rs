use crate::error::DocGenResult;
use crate::lang;
use crate::lang::LanguageParser;

/// Represents a code item that needs documentation
#[derive(Debug, Clone)]
pub struct CodeItem {
    pub item_type: String,        // "function", "method", "class", etc.
    pub name: String,             // Name of the function/class/method
    pub line_number: usize,       // Line number in the file
    pub code: String,             // The code for this item
    pub existing_docstring: Option<String>, // Existing docstring, if any
    pub parent: Option<String>,   // Parent type (e.g., class for methods)
    pub parameters: Vec<String>,  // Function/method parameters
    pub returns: Option<String>,  // Return type annotation if available
    pub indentation: String,      // Indentation used for this item
}

/// Represents the parsed code file
#[derive(Debug)]
pub struct ParsedCode {
    pub items: Vec<CodeItem>,
    pub original_content: String,
}

/// Parse a Python file and extract code items
pub fn parse_python(content: &str) -> DocGenResult<ParsedCode> {
    let parser = lang::python::PythonParser::new();
    parser.parse(content)
}

/// Parse a Rust file and extract code items
/// 
/// Note: Currently disabled until tree-sitter issues are resolved.
/// All calls will redirect to the Python parser for now.
pub fn parse_rust(content: &str) -> DocGenResult<ParsedCode> {
    // Temporarily using Python parser until Rust parser is fixed
    println!("Warning: Rust parser is not yet implemented. Using Python parser instead.");
    parse_python(content)
    // Uncomment when ready:
    // let parser = lang::rust::RustParser::new();
    // parser.parse(content)
}

/// Parse a JavaScript file and extract code items
/// 
/// Note: Currently disabled until tree-sitter issues are resolved.
/// All calls will redirect to the Python parser for now.
pub fn parse_javascript(content: &str) -> DocGenResult<ParsedCode> {
    // Temporarily using Python parser until JavaScript parser is fixed
    println!("Warning: JavaScript parser is not yet implemented. Using Python parser instead.");
    parse_python(content)
    // Uncomment when ready:
    // let parser = lang::javascript::JavaScriptParser::new();
    // parser.parse(content)
}

/// Parse a TypeScript file and extract code items
/// 
/// Note: Currently disabled until tree-sitter issues are resolved.
/// All calls will redirect to the Python parser for now.
pub fn parse_typescript(content: &str) -> DocGenResult<ParsedCode> {
    // Temporarily using Python parser until TypeScript parser is fixed
    println!("Warning: TypeScript parser is not yet implemented. Using Python parser instead.");
    parse_python(content)
    // Uncomment when ready:
    // let parser = lang::typescript::TypeScriptParser::new();
    // parser.parse(content)
}
