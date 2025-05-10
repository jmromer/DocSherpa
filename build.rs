// build.rs
use std::path::PathBuf;

fn main() {
    // Tell Cargo to rerun this script if any of these files change
    println!("cargo:rerun-if-changed=build.rs");
    
    // Python support is provided by rustpython-parser crate, so we don't need to build 
    // tree-sitter-python grammar ourselves
    
    // Build tree-sitter-rust
    let rust_dir = PathBuf::from("./vendor/tree-sitter-rust");
    if !rust_dir.exists() {
        std::fs::create_dir_all(&rust_dir).unwrap();
        println!("cargo:warning=Created directory for tree-sitter-rust");
    }
    
    // Build tree-sitter-javascript
    let js_dir = PathBuf::from("./vendor/tree-sitter-javascript");
    if !js_dir.exists() {
        std::fs::create_dir_all(&js_dir).unwrap();
        println!("cargo:warning=Created directory for tree-sitter-javascript");
    }
    
    // Build tree-sitter-typescript
    let ts_dir = PathBuf::from("./vendor/tree-sitter-typescript");
    if !ts_dir.exists() {
        std::fs::create_dir_all(&ts_dir).unwrap();
        println!("cargo:warning=Created directory for tree-sitter-typescript");
    }
    
    // Link to the language libraries
    println!("cargo:rustc-link-lib=tree-sitter");
    
    // All language links are disabled until we properly set up the tree-sitter parsers
    // For future reference, when enabling languages, uncomment the appropriate lines below
    // println!("cargo:rustc-link-lib=static=tree_sitter_javascript");
    // println!("cargo:rustc-link-lib=static=tree_sitter_rust");
    // println!("cargo:rustc-link-lib=static=tree_sitter_python");
    // println!("cargo:rustc-link-lib=static=tree_sitter_typescript");
    
    // Note to users: For now, we're using pre-built grammar files from the crates
    println!("cargo:warning=Using pre-built tree-sitter grammars from their respective crates");
    println!("cargo:warning=If you encounter linking errors, you might need to install the tree-sitter CLI");
    println!("cargo:warning=and manually build the grammar files in the vendor directory");
}