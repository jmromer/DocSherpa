use crate::error::DocGenResult;
use crate::parser::{ParsedCode, CodeItem};

/// Represents an issue with documentation
#[derive(Debug)]
pub struct DocstringIssue {
    pub item_type: String,      // "function", "method", "class"
    pub name: String,           // Name of the item
    pub line_number: usize,     // Line number in the file
    pub issue_type: String,     // "missing" or "outdated"
    pub item_index: usize,      // Index in the parsed items array
}

/// Analyze parsed code for docstring issues
pub fn analyze(parsed_code: &ParsedCode) -> DocGenResult<Vec<DocstringIssue>> {
    let mut issues = Vec::new();
    
    for (index, item) in parsed_code.items.iter().enumerate() {
        // Check if docstring is missing
        if item.existing_docstring.is_none() {
            issues.push(DocstringIssue {
                item_type: item.item_type.clone(),
                name: item.name.clone(),
                line_number: item.line_number,
                issue_type: "missing".to_string(),
                item_index: index,
            });
            continue;
        }
        
        // Check if docstring might be outdated
        // This is a simplistic check that can be enhanced
        if let Some(docstring) = &item.existing_docstring {
            if is_likely_outdated(item, docstring) {
                issues.push(DocstringIssue {
                    item_type: item.item_type.clone(),
                    name: item.name.clone(),
                    line_number: item.line_number,
                    issue_type: "outdated".to_string(),
                    item_index: index,
                });
            }
        }
    }
    
    Ok(issues)
}

/// Check if a docstring is likely outdated
/// This is a simple heuristic and can be enhanced
fn is_likely_outdated(item: &CodeItem, docstring: &str) -> bool {
    // Check if all parameters are mentioned in the docstring
    for param in &item.parameters {
        // Skip self parameter for methods
        if param == "self" {
            continue;
        }
        
        // Clean parameter name for comparison (remove * or = if present)
        let clean_param = param.trim_matches(|c| c == '*' || c == '=');
        
        // Check if parameter is mentioned in docstring
        if !docstring.contains(clean_param) {
            return true;
        }
    }
    
    // Check if return type is mentioned for functions/methods with return annotations
    if let Some(_) = &item.returns {
        if !docstring.to_lowercase().contains("return") {
            return true;
        }
    }
    
    // Check if docstring is very short (likely a placeholder)
    if docstring.trim().len() < 10 {
        return true;
    }
    
    false
}

/// Represents an updated docstring
#[derive(Debug, Clone)]
pub struct UpdatedDocstring {
    pub item_index: usize,
    pub new_docstring: String,
    pub indentation: String,
}
