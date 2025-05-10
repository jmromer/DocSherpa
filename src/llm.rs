use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::config::Config;
use crate::docstring::{DocstringIssue, UpdatedDocstring};
use crate::error::{DocGenError, DocGenResult};
use crate::parser::ParsedCode;

/// Trait for LLM clients
#[async_trait]
pub trait LlmClient {
    async fn generate_docstrings(
        &self, 
        parsed_code: &ParsedCode, 
        issues: &[DocstringIssue]
    ) -> DocGenResult<Vec<UpdatedDocstring>>;
}

/// Factory function to get the appropriate LLM client
pub fn get_client(provider: &str) -> DocGenResult<Box<dyn LlmClient>> {
    // For the "mock" provider, return our mock client for testing
    if provider.to_lowercase() == "mock" {
        return Ok(Box::new(MockLlmClient::new()));
    }
    
    match provider.to_lowercase().as_str() {
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| DocGenError::ConfigError("OPENAI_API_KEY environment variable is not set".into()))?;
            Ok(Box::new(OpenAiClient::new(api_key)))
        },
        "claude" => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| DocGenError::ConfigError("ANTHROPIC_API_KEY environment variable is not set".into()))?;
            Ok(Box::new(ClaudeClient::new(api_key)))
        },
        _ => Err(DocGenError::ConfigError(format!("Unsupported LLM provider: {}", provider))),
    }
}

/// OpenAI client implementation
pub struct OpenAiClient {
    api_key: String,
    client: Client,
}

impl OpenAiClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        
        Self { api_key, client }
    }
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn generate_docstrings(
        &self, 
        parsed_code: &ParsedCode, 
        issues: &[DocstringIssue]
    ) -> DocGenResult<Vec<UpdatedDocstring>> {
        let mut updated_docstrings = Vec::new();
        
        for issue in issues {
            let item = &parsed_code.items[issue.item_index];
            
            // Prepare prompt
            let prompt = format!(
                "Generate a Python docstring for the following {} '{}'. \
                Follow PEP 257 style guidelines.\
                The docstring should be informative, accurate, and describe what the {} does.\
                Include parameters, return values, and exceptions if applicable.\
                Return ONLY the docstring text without the triple quotes or indentation.\n\n\
                ```python\n{}\n```",
                item.item_type, item.name, item.item_type, item.code
            );
            
            // Make API request
            let response = self.client.post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "model": "gpt-4",
                    "messages": [
                        {
                            "role": "system",
                            "content": "You are a Python documentation assistant. Generate clear, concise, and accurate docstrings for Python code."
                        },
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ],
                    "temperature": 0.3,
                    "max_tokens": 1000
                }))
                .send()
                .await
                .map_err(|e| DocGenError::LlmApiError(e.to_string()))?;
            
            // Parse response
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(DocGenError::LlmApiError(format!("API request failed: {}", error_text)));
            }
            
            let response_json: OpenAiResponse = response.json().await
                .map_err(|e| DocGenError::LlmApiError(format!("Failed to parse API response: {}", e)))?;
            
            if response_json.choices.is_empty() {
                return Err(DocGenError::LlmApiError("API response contained no choices".into()));
            }
            
            let docstring_text = response_json.choices[0].message.content.trim();
            
            // Format the docstring with triple quotes and proper indentation
            let formatted_docstring = format!("\"\"\"{}\"\"\"", docstring_text);
            
            updated_docstrings.push(UpdatedDocstring {
                item_index: issue.item_index,
                new_docstring: formatted_docstring,
                indentation: item.indentation.clone(),
            });
        }
        
        Ok(updated_docstrings)
    }
}

/// Claude client implementation
pub struct ClaudeClient {
    api_key: String,
    client: Client,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        
        Self { api_key, client }
    }
}

/// Mock LLM client for testing without API calls
pub struct MockLlmClient;

impl MockLlmClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn generate_docstrings(
        &self, 
        parsed_code: &ParsedCode, 
        issues: &[DocstringIssue]
    ) -> DocGenResult<Vec<UpdatedDocstring>> {
        let mut updated_docstrings = Vec::new();
        
        for issue in issues {
            let item = &parsed_code.items[issue.item_index];
            
            // Generate a mock docstring based on item type
            let mock_docstring = match item.item_type.as_str() {
                "function" => {
                    let params = item.parameters.join(", ");
                    format!("Mock docstring for function {}.\nParameters: {}", item.name, params)
                },
                "class" => {
                    format!("Mock docstring for class {}.", item.name)
                },
                "method" => {
                    let params = item.parameters.join(", ");
                    format!("Mock docstring for method {}.\nParameters: {}", item.name, params)
                },
                _ => format!("Mock docstring for {} {}.", item.item_type, item.name),
            };
            
            // Format the docstring with triple quotes
            let formatted_docstring = format!("\"\"\"{}\"\"\"", mock_docstring);
            
            updated_docstrings.push(UpdatedDocstring {
                item_index: issue.item_index,
                new_docstring: formatted_docstring,
                indentation: item.indentation.clone(),
            });
        }
        
        Ok(updated_docstrings)
    }
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn generate_docstrings(
        &self, 
        parsed_code: &ParsedCode, 
        issues: &[DocstringIssue]
    ) -> DocGenResult<Vec<UpdatedDocstring>> {
        let mut updated_docstrings = Vec::new();
        
        for issue in issues {
            let item = &parsed_code.items[issue.item_index];
            
            // Prepare prompt
            let prompt = format!(
                "Generate a Python docstring for the following {} '{}'. \
                Follow PEP 257 style guidelines.\
                The docstring should be informative, accurate, and describe what the {} does.\
                Include parameters, return values, and exceptions if applicable.\
                Return ONLY the docstring text without the triple quotes or indentation.\n\n\
                ```python\n{}\n```",
                item.item_type, item.name, item.item_type, item.code
            );
            
            // Make API request
            let response = self.client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&json!({
                    "model": "claude-3-opus-20240229",
                    "max_tokens": 1000,
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                }))
                .send()
                .await
                .map_err(|e| DocGenError::LlmApiError(e.to_string()))?;
            
            // Parse response
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(DocGenError::LlmApiError(format!("API request failed: {}", error_text)));
            }
            
            let response_json: ClaudeResponse = response.json().await
                .map_err(|e| DocGenError::LlmApiError(format!("Failed to parse API response: {}", e)))?;
            
            if response_json.content.is_empty() {
                return Err(DocGenError::LlmApiError("API response contained no content".into()));
            }
            
            let docstring_text = response_json.content[0].text.trim();
            
            // Format the docstring with triple quotes and proper indentation
            let formatted_docstring = format!("\"\"\"{}\"\"\"", docstring_text);
            
            updated_docstrings.push(UpdatedDocstring {
                item_index: issue.item_index,
                new_docstring: formatted_docstring,
                indentation: item.indentation.clone(),
            });
        }
        
        Ok(updated_docstrings)
    }
}
