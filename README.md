# DocGen: Documentation Generator

DocGen is a command-line tool written in Rust that automatically detects missing or outdated documentation in code files and uses LLM APIs to generate or update docstrings. It currently supports Python with JavaScript and Rust support in development.

## Features

- Automatically detects missing docstrings in functions, methods, and classes
- Identifies outdated docstrings that need to be updated
- Uses OpenAI's API to generate accurate, informative docstrings
- Follows documentation style guidelines for each language
- Supports multiple programming languages with a language-agnostic architecture
- Preserves proper indentation and formatting in the updated files

## Current Status

This tool is in active development. Current implementation status:

- ✅ Python support is complete and fully functional
- 🔄 JavaScript support is partially implemented
- 🔄 Rust support is partially implemented
- 🔄 TypeScript support planned

## Prerequisites

- Rust and Cargo (2021 edition or later)
- OpenAI API key for generating docstrings

## Installation

### From Source

1. Clone the repository:

```bash
git clone https://github.com/jmromer/docgen.git
cd docgen
```

2. Build the application:

```bash
cargo build --release
```

3. The compiled binary will be available at `target/release/docgen`

## Configuration

Create a `.env` file in your project directory with your OpenAI API key:

```
OPENAI_API_KEY=your_api_key_here
```

## Usage

### Basic Usage

```bash
docgen <file_paths>
```

Example:
```bash
docgen src/main.py src/utils.py
```

### Command Line Options

```
Usage: docgen [OPTIONS] <FILES>...

Arguments:
  <FILES>...  Files to process

Options:
  -l, --language <LANGUAGE>  Programming language mode [default: auto]
                             Possible values:
                             - python: Python language support
                             - rust: Rust language support
                             - javascript: JavaScript language support
                             - typescript: TypeScript language support
                             - auto: Automatically detect based on file extension
  -p, --provider <PROVIDER>  LLM provider to use [default: openai]
                             Possible values:
                             - openai: Use OpenAI API
                             - mock: Use mock provider for testing
  -c, --check                Check mode - only report issues without making changes
  -v, --verbose              Verbose mode - show more details
      --test                 Test mode - analyze files without making API calls
  -h, --help                 Print help
  -V, --version              Print version
```

### Example Commands

Check for missing docstrings without making changes:
```bash
docgen --check src/main.py
```

Generate docstrings in verbose mode:
```bash
docgen --verbose src/main.py
```

Test the parser without making API calls:
```bash
docgen --test src/main.py
```

Specify a language explicitly:
```bash
docgen --language python src/main.py
```

### Mock Provider for Testing

If you want to test the functionality without using the OpenAI API:

```bash
docgen --provider mock src/main.py
```

## Language Support

### Python

Full support with PEP 257 style docstrings. Example:

```python
def calculate_sum(a, b):
    """Calculate the sum of two numbers.
    
    Parameters:
    a (int or float): The first number to be added.
    b (int or float): The second number to be added.
    
    Returns:
    int or float: The sum of the two input numbers.
    
    Raises:
    TypeError: If the inputs are not integers or floats.
    """
    if not (isinstance(a, (int, float)) and isinstance(b, (int, float))):
        raise TypeError("Both inputs must be integers or floats")
    return a + b
```

### JavaScript (In Development)

Basic support for JSDoc style comments:

```javascript
/**
 * Calculates the sum of two numbers.
 *
 * @param {number} a - The first number to be added.
 * @param {number} b - The second number to be added.
 * @returns {number} The sum of the two input numbers.
 * @throws {TypeError} If the inputs are not numbers.
 */
function calculateSum(a, b) {
    if (typeof a !== 'number' || typeof b !== 'number') {
        throw new TypeError("Both inputs must be numbers");
    }
    return a + b;
}
```

### Rust (In Development)

Basic support for Rust documentation comments:

```rust
/// Calculates the sum of two numbers.
///
/// # Arguments
///
/// * `a` - The first number to be added.
/// * `b` - The second number to be added.
///
/// # Returns
///
/// The sum of the two input numbers.
///
/// # Errors
///
/// None
fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
```

## Project Structure

```
docgen/
├── src/
│   ├── lang/               # Language-specific parsers
│   │   ├── python.rs       # Python parser implementation 
│   │   ├── rust.rs         # Rust parser implementation
│   │   ├── javascript.rs   # JavaScript parser implementation
│   │   ├── typescript.rs   # TypeScript parser implementation
│   │   └── mod.rs          # Language module definitions
│   ├── config.rs           # Configuration handling
│   ├── docstring.rs        # Docstring representation
│   ├── error.rs            # Error handling
│   ├── llm.rs              # LLM API client implementations
│   ├── main.rs             # CLI entry point
│   ├── parser.rs           # Generic code parsing
│   └── updater.rs          # File update operations
├── Cargo.toml              # Project dependencies
└── README.md               # This file
```

## Architecture

The tool is designed with a language-agnostic architecture:

1. **Parser Layer**: Implements the `LanguageParser` trait for each supported language to extract code items and their existing docstrings.

2. **Analysis Layer**: Determines which functions, methods, or classes need docstrings generated or updated.

3. **LLM Integration**: Uses OpenAI's API to generate appropriate docstrings based on the code context.

4. **Update Layer**: Inserts or updates docstrings with proper formatting and indentation.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding a New Language

To add support for a new language:

1. Add a new parser implementation in `src/lang/` 
2. Implement the `LanguageParser` trait for the new language
3. Update the `get_parser` function in `src/lang/mod.rs`
4. Add the language to the CLI options in `main.rs`
5. Add appropriate unit tests to verify functionality

## License

This project is licensed under the MIT License.