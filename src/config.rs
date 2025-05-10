/// Configuration for DocGen
pub struct Config {
    /// LLM provider to use (openai or claude)
    pub provider: String,
    
    /// Check mode - only report issues without making changes
    pub check_only: bool,
    
    /// Verbose mode - show more details
    pub verbose: bool,
    
    /// Test mode - analyze files without making API calls
    pub test_mode: bool,
}

impl Config {
    pub fn get_api_key(&self) -> Option<String> {
        match self.provider.to_lowercase().as_str() {
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "claude" => std::env::var("ANTHROPIC_API_KEY").ok(),
            _ => None,
        }
    }
}
