use crate::docstring::UpdatedDocstring;
use crate::error::DocGenResult;
use crate::lang;

/// Update the file content with new docstrings
pub fn update_file_content(
    original_content: &str,
    language: &super::Language,
    updated_docstrings: &[UpdatedDocstring],
) -> DocGenResult<String> {
    let parser = lang::get_parser(language);
    parser.update_content(original_content, updated_docstrings)
}
