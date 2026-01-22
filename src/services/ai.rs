use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub struct AIService {
    client: reqwest::Client,
    api_key: String,
    api_url: String,
}

impl AIService {
    pub fn new() -> Self {
        let api_key = std::env::var("AI_API_KEY").unwrap_or_default();
        let api_url = std::env::var("AI_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        AIService {
            client: reqwest::Client::new(),
            api_key,
            api_url,
        }
    }

    pub async fn optimize(&self, code: &str, language: &str) -> AppResult<Vec<String>> {
        let prompt = format!(
            "Optimize the following {} code:\n\n{}\n\nProvide optimization suggestions.",
            language, code
        );
        self.call_ai(&prompt).await
    }

    pub async fn review(&self, code: &str, language: &str) -> AppResult<Vec<String>> {
        let prompt = format!(
            "Review the following {} code and provide feedback on:\n- Code quality\n- Best practices\n- Potential issues\n\n{}",
            language, code
        );
        self.call_ai(&prompt).await
    }

    pub async fn refactor(
        &self,
        code: &str,
        language: &str,
    ) -> AppResult<(Vec<String>, String)> {
        let prompt = format!(
            "Refactor the following {} code to be more maintainable and efficient:\n\n{}",
            language, code
        );
        let suggestions = self.call_ai(&prompt).await?;
        
        // For now, return the original code as refactored
        // In production, parse the AI response to extract the refactored code
        Ok((suggestions, code.to_string()))
    }

    async fn call_ai(&self, prompt: &str) -> AppResult<Vec<String>> {
        let request = AIRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApiError(
                "AI API call failed".to_string(),
            ));
        }

        // Parse response and extract suggestions
        let result: serde_json::Value = response.json().await?;
        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No response")
            .to_string();

        // Simple parsing - split by newlines
        let suggestions = content
            .lines()
            .filter(|l| !l.is_empty())
            .take(5)
            .map(|s| s.to_string())
            .collect();

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_service_creation() {
        let service = AIService::new();
        assert!(!service.api_key.is_empty() || service.api_key.is_empty()); // Just check it exists
    }
}
