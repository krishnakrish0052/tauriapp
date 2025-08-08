use reqwest::Client;
use serde::Deserialize;
use anyhow::Result;
use log::{info, error};
use futures_util::stream::StreamExt;
use serde_json::Value;

// Model enum for Pollinations models
#[derive(Debug, Clone)]
pub enum PollinationsModel {
    Custom(String),
}

impl PollinationsModel {
    pub fn as_str(&self) -> &str {
        match self {
            PollinationsModel::Custom(s) => s.as_str(),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            PollinationsModel::Custom(s) => {
                match s.as_str() {
                    "llama-fast-roblox" => "Llama Fast Roblox",
                    "llama-roblox" => "Llama Roblox", 
                    "mistral" => "Mistral",
                    "openai" => "OpenAI GPT-4",
                    _ => s.as_str(),
                }
            }
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(PollinationsModel::Custom(s.to_string()))
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(PollinationsModel::Custom(s.to_string()))
    }
}

// Model info from API response
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
}

// Pollinations client
#[derive(Clone)]
pub struct PollinationsClient {
    client: Client,
    base_url: String,
}

impl PollinationsClient {
    pub fn new(_api_key: String, _referer: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://text.pollinations.ai".to_string(),
        }
    }

    pub async fn generate_answer(
        &self,
        question: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
    ) -> Result<String> {
        let system_prompt = self.build_system_prompt(context);
        let prompt = format!("{}

Interview Question: {}

Please provide a comprehensive and professional answer to this interview question.", system_prompt, question);

        info!("Generating answer with Pollinations model: {}", model.as_str());
        
        // Build URL with proper query parameters
        let mut url = reqwest::Url::parse(&format!("{}/", self.base_url))?;
        url.query_pairs_mut()
            .append_pair("prompt", &prompt)
            .append_pair("model", model.as_str())
            .append_pair("private", "true")
            .append_pair("referrer", "mockmate")
            .append_pair("temperature", "0.7");

        info!("Pollinations request URL: {}", url);
        
        let response = self
            .client
            .get(url)
            .header("User-Agent", "MockMate/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Pollinations API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations API error {}: {}", status, error_text));
        }

        let response_text = response.text().await?;
        info!("Received response from Pollinations: {} characters", response_text.len());
        
        if response_text.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations API"))
        } else {
            Ok(response_text.trim().to_string())
        }
    }

    fn build_system_prompt(&self, context: &super::openai::InterviewContext) -> String {
        let mut prompt = String::from("You are an AI interview assistant helping a candidate prepare for their interview. Provide clear, professional, and comprehensive answers to interview questions.");

        if let Some(company) = &context.company {
            prompt.push_str(&format!("\n\nCompany: {}", company));
        }

        if let Some(position) = &context.position {
            prompt.push_str(&format!("\nPosition: {}", position));
        }

        if let Some(job_description) = &context.job_description {
            prompt.push_str(&format!("\nJob Description: {}", job_description));
        }

        prompt.push_str("\n\nGuidelines for your responses:");
        prompt.push_str("\n- Provide structured, well-organized answers");
        prompt.push_str("\n- Include specific examples when relevant");
        prompt.push_str("\n- Keep responses professional and concise");
        prompt.push_str("\n- Focus on demonstrating relevant skills and experience");
        prompt.push_str("\n- Use the STAR method (Situation, Task, Action, Result) for behavioral questions when appropriate");

        prompt
    }

    // New method: Generate answer with streaming using SSE
    pub async fn generate_answer_streaming<F>(
        &self,
        question: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        let system_prompt = self.build_system_prompt(context);
        let prompt = format!("{}

Interview Question: {}

Please provide a comprehensive and professional answer to this interview question.", system_prompt, question);

        info!("Generating streaming answer with Pollinations model: {}", model.as_str());
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        // Build URL with streaming enabled
        let encoded_prompt = urlencoding::encode(&prompt);
        let url = format!(
            "{}/{}?model={}&stream=true&private=true&referrer={}&temperature=0.7",
            self.base_url, encoded_prompt, model.as_str(), referrer
        );

        info!("Pollinations streaming request URL: {}", url);
        
        let mut request_builder = self.client.get(&url)
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache");

        // Add Bearer token if available
        if !api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Pollinations streaming API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations streaming API error {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);
                    
                    // Process SSE lines
                    let lines: Vec<&str> = buffer.lines().collect();
                    if lines.len() > 1 {
                        // Process all complete lines except the last (incomplete) one
                        for line in &lines[..lines.len()-1] {
                            if let Some(content) = self.parse_sse_line(line) {
                                if content == "[DONE]" {
                                    info!("SSE stream completed");
                                    return Ok(full_response);
                                }
                                
                                // Call the callback with the new content
                                on_token(&content);
                                full_response.push_str(&content);
                            }
                        }
                        // Keep the last incomplete line in buffer
                        buffer = lines.last().unwrap_or(&"").to_string();
                    }
                }
                Err(e) => {
                    error!("Error reading SSE stream: {}", e);
                    break;
                }
            }
        }
        
        // Process any remaining content in buffer
        if !buffer.is_empty() {
            if let Some(content) = self.parse_sse_line(&buffer) {
                if content != "[DONE]" {
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations streaming API"))
        } else {
            Ok(full_response.trim().to_string())
        }
    }

    // Helper method to parse SSE lines
    fn parse_sse_line(&self, line: &str) -> Option<String> {
        let line = line.trim();
        
        // Handle SSE data lines
        if line.starts_with("data: ") {
            let json_str = &line[6..]; // Remove "data: " prefix
            
            // Check for completion marker
            if json_str.trim() == "[DONE]" {
                return Some("[DONE]".to_string());
            }
            
            // Try to parse as JSON for OpenAI-compatible format
            match serde_json::from_str::<Value>(json_str) {
                Ok(json) => {
                    // Look for content in various possible structures
                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                        if let Some(first_choice) = choices.first() {
                            if let Some(delta) = first_choice.get("delta") {
                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                    return Some(content.to_string());
                                }
                            }
                        }
                    }
                    
                    // Fallback: look for direct text field
                    if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
                Err(_) => {
                    // If not JSON, treat the content after "data: " as raw text
                    return Some(json_str.to_string());
                }
            }
        }
        
        // Handle non-SSE format - if the line looks like text content directly
        else if !line.is_empty() && !line.starts_with(":") && !line.starts_with("event:") {
            return Some(line.to_string());
        }
        
        None
    }

    // New method: Generate answer using POST endpoint with OpenAI compatibility
    pub async fn generate_answer_post(
        &self,
        question: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
        stream: bool,
    ) -> Result<reqwest::Response> {
        let system_prompt = self.build_system_prompt(context);
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": question
            })
        ];

        let mut payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": stream,
            "private": true,
            "temperature": 0.7
        });

        // Add referrer to payload if available
        if !referrer.is_empty() {
            payload["referrer"] = serde_json::Value::String(referrer);
        }

        info!("Generating answer with POST endpoint: model={}, stream={}", model.as_str(), stream);
        
        let url = format!("{}/openai", self.base_url);
        let mut request_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0");

        if stream {
            request_builder = request_builder.header("Accept", "text/event-stream");
        }

        // Add Bearer token if available
        if !api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Pollinations POST API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations POST API error {}: {}", status, error_text));
        }

        Ok(response)
    }

    // New method: Generate streaming answer with POST endpoint
    pub async fn generate_answer_post_streaming<F>(
        &self,
        question: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        let response = self.generate_answer_post(question, context, model, true).await?;
        
        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);
                    
                    // Process SSE lines
                    let lines: Vec<&str> = buffer.lines().collect();
                    if lines.len() > 1 {
                        for line in &lines[..lines.len()-1] {
                            if let Some(content) = self.parse_sse_line(line) {
                                if content == "[DONE]" {
                                    info!("POST SSE stream completed");
                                    return Ok(full_response);
                                }
                                
                                on_token(&content);
                                full_response.push_str(&content);
                            }
                        }
                        buffer = lines.last().unwrap_or(&"").to_string();
                    }
                }
                Err(e) => {
                    error!("Error reading POST SSE stream: {}", e);
                    break;
                }
            }
        }
        
        // Process remaining content
        if !buffer.is_empty() {
            if let Some(content) = self.parse_sse_line(&buffer) {
                if content != "[DONE]" {
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations POST streaming API"))
        } else {
            Ok(full_response.trim().to_string())
        }
    }

    pub async fn fetch_available_models(&self) -> Result<Vec<PollinationsModel>> {
        let url = format!("{}/models", self.base_url);
        info!("Fetching models from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "MockMate/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            error!("Failed to fetch models: HTTP {} - {}", status, err_text);
            return self.get_fallback_models();
        }

        let response_text = response.text().await?;
        info!("Raw models response length: {} chars", response_text.len());
        
        // Try to parse as JSON array of model objects
        match serde_json::from_str::<Vec<ModelInfo>>(&response_text) {
            Ok(model_infos) => {
                let mut models: Vec<PollinationsModel> = Vec::new();
                for model_info in model_infos {
                    models.push(PollinationsModel::Custom(model_info.name));
                }
                info!("Successfully parsed {} models from API", models.len());
                Ok(models)
            }
            Err(e) => {
                error!("Failed to parse models JSON: {}", e);
                self.get_fallback_models()
            }
        }
    }
    
    fn get_fallback_models(&self) -> Result<Vec<PollinationsModel>> {
        info!("Using fallback models list");
        let models = vec![
            PollinationsModel::Custom("llama-fast-roblox".to_string()),
            PollinationsModel::Custom("llama-roblox".to_string()),
            PollinationsModel::Custom("mistral".to_string()),
            PollinationsModel::Custom("openai".to_string()),
        ];
        Ok(models)
    }
}

// AI Provider enum to distinguish between OpenAI and Pollinations
#[derive(Debug, Clone, PartialEq)]
pub enum AIProvider {
    OpenAI,
    Pollinations,
}

impl AIProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(AIProvider::OpenAI),
            "pollinations" | "self" => Some(AIProvider::Pollinations),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AIProvider::OpenAI => "openai",
            AIProvider::Pollinations => "pollinations",
        }
    }
}
