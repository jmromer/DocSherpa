pub mod python;
// Temporarily disabled until tree-sitter linking issues are resolved
// pub mod rust;
// pub mod javascript;
// pub mod typescript;

/// Trait for language-specific code structure parsers
pub trait LanguageParser {
    /// Parse code content into a structured representation
    fn parse(&self, content: &str) -> crate::error::DocGenResult<crate::parser::ParsedCode>;
    
    /// Update a file's content with new docstrings
    fn update_content(
        &self, 
        content: &str, 
        updated_docstrings: &[crate::docstring::UpdatedDocstring]
    ) -> crate::error::DocGenResult<String>;
}

/// Factory function to get a language parser implementation
pub fn get_parser(language: &super::Language) -> Box<dyn LanguageParser> {
    match language {
        // For now, only Python is fully implemented
        super::Language::Python => Box::new(python::PythonParser::new()),
        // Other languages temporarily return Python parser until tree-sitter is fixed
        _ => {
            println!("Warning: Requested language not fully implemented. Using Python parser instead.");
            Box::new(python::PythonParser::new())
        }
        // Uncomment these when tree-sitter linking issues are resolved
        // super::Language::Rust => Box::new(rust::RustParser::new()),
        // super::Language::JavaScript => Box::new(javascript::JavaScriptParser::new()),
        // super::Language::TypeScript => Box::new(typescript::TypeScriptParser::new()),
    }
}