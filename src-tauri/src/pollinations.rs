use reqwest::Client;
use serde::Deserialize;
use anyhow::Result;
use log::{info, error, warn, debug};
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
                    "openai" => "OpenAI GPT-4",
                    "mistral" => "Mistral",
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
    api_key: String,
    referrer: String,
}

impl PollinationsClient {
    pub fn new(api_key: String, referrer: String) -> Self {
        // Ultra-fast HTTP client configuration for maximum speed
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))   // Much faster timeout
            .connect_timeout(std::time::Duration::from_secs(1))  // Ultra-fast connection
            .tcp_keepalive(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(5))
            .pool_max_idle_per_host(20)  // More connections for speed
            .http2_keep_alive_interval(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        Self {
            client,
            base_url: "https://text.pollinations.ai".to_string(),
            api_key,
            referrer,
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

Provide a confident, direct, and authentic answer that demonstrates your qualifications. Keep it focused and conversational - aim for 30-60 seconds when spoken aloud. Be specific and impactful.", system_prompt, question);

        info!("Generating answer with Pollinations model: {}", model.as_str());
        
        // Try different endpoints and formats for Pollinations API
        let endpoints = vec![
            ("https://text.pollinations.ai/openai", "json"),
            ("https://text.pollinations.ai", "text"),
        ];
        
        for (base_url, response_format) in endpoints {
            info!("Trying Pollinations endpoint: {} (format: {})", base_url, response_format);
            
            match self.try_generate_with_endpoint(base_url, &prompt, &model, response_format).await {
                Ok(result) => {
                    info!("✅ Successfully generated answer with endpoint: {}", base_url);
                    return Ok(result);
                }
                Err(e) => {
                    error!("❌ Failed with endpoint {}: {}", base_url, e);
                    continue;
                }
            }
        }
        
        // If all endpoints fail, return a helpful error
        Err(anyhow::anyhow!("All Pollinations API endpoints failed. The service might be unavailable."))
    }
    
    async fn try_generate_with_endpoint(
        &self,
        base_url: &str,
        prompt: &str,
        model: &PollinationsModel,
        response_format: &str,
    ) -> Result<String> {
        match response_format {
            "json" => self.try_json_endpoint(base_url, prompt, model).await,
            "text" => self.try_text_endpoint(base_url, prompt, model).await,
            _ => Err(anyhow::anyhow!("Unknown response format: {}", response_format))
        }
    }
    
    async fn try_json_endpoint(
        &self,
        base_url: &str,
        prompt: &str,
        model: &PollinationsModel,
    ) -> Result<String> {
        let payload = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful AI assistant."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "model": model.as_str(),
            "stream": false,
            "temperature": 0.7
        });
        
        // Add referrer to payload for seed tier access
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());
        let mut final_payload = payload;
        if !referrer.is_empty() {
            final_payload["referrer"] = serde_json::Value::String(referrer.clone());
        }

        let response = self
            .client
            .post(base_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0")
            .header("Referer", referrer.as_str())  // Add referrer header for seed tier
            .json(&final_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_text));
        }

        let response_text = response.text().await?;
        
        // Try to parse as JSON first
        if let Ok(json_response) = serde_json::from_str::<Value>(&response_text) {
            // Extract content from different possible JSON structures
            if let Some(content) = json_response.get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str()) {
                return Ok(content.to_string());
            }
            
            if let Some(content) = json_response.get("response")
                .and_then(|response| response.as_str()) {
                return Ok(content.to_string());
            }
            
            if let Some(content) = json_response.get("content")
                .and_then(|content| content.as_str()) {
                return Ok(content.to_string());
            }
        }
        
        // If JSON parsing fails or doesn't contain expected fields, return raw text
        if response_text.trim().starts_with("<!DOCTYPE html>") || response_text.trim().starts_with("<html") {
            return Err(anyhow::anyhow!("Received HTML response instead of JSON"));
        }
        
        Ok(response_text)
    }
    
    async fn try_text_endpoint(
        &self,
        base_url: &str,
        prompt: &str,
        model: &PollinationsModel,
    ) -> Result<String> {
        // Build URL with proper query parameters for text endpoint
        let mut url = reqwest::Url::parse(&format!("{}/", base_url))?;
        url.query_pairs_mut()
            .append_pair("prompt", prompt)
            .append_pair("model", model.as_str())
            .append_pair("private", "true")
            .append_pair("referrer", "mockmate")
            .append_pair("temperature", "0.1")  // Ultra-low temperature for maximum speed
            .append_pair("max_tokens", "80");   // Ultra-small for lightning speed

        info!("Pollinations text request URL: {}", url.as_str().chars().take(200).collect::<String>() + "...");
        
        let response = self
            .client
            .get(url)
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "text/plain")
            .header("Referer", "mockmate")  // Add referrer header for seed tier
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_text));
        }

        let response_text = response.text().await?;
        info!("Received response from Pollinations text endpoint: {} characters", response_text.len());
        
        // Check if we got HTML instead of plain text
        if response_text.trim().starts_with("<!DOCTYPE html>") || response_text.trim().starts_with("<html") {
            return Err(anyhow::anyhow!("Received HTML response instead of plain text"));
        }
        
        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response from Pollinations API"));
        }
        
        Ok(response_text.trim().to_string())
    }

    /// Analyze a screenshot using Pollinations with vision-capable models (like OpenAI GPT-4)
    pub async fn analyze_screenshot_with_vision(
        &self,
        base64_image: &str,
        analysis_prompt: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
    ) -> Result<String> {
        info!("🔍 Analyzing screenshot with Pollinations model: {}", model.as_str());
        
        // Build a comprehensive system prompt for screenshot analysis
        let system_prompt = format!(
            "You are an expert technical interviewer analyzing a screenshot to generate relevant interview questions. {}

{}

IMPORTANT INSTRUCTIONS:
1. Carefully examine the screenshot to identify technical content
2. Generate ONE specific, relevant interview question based on what you see
3. The question should test the candidate's understanding of the visible content
4. Return your response in this EXACT JSON format:
{{
  \"question\": \"Your specific interview question here?\",
  \"analysis\": \"Brief explanation of why this question is relevant\",
  \"confidence\": 0.9
}}

The screenshot might contain:
- Source code (any programming language)
- Development tools and IDEs (VS Code, IntelliJ, etc.)
- Documentation or technical articles
- System interfaces or applications
- Technical diagrams or architecture
- Database schemas or queries
- Configuration files (JSON, YAML, XML)
- Terminal/command line interfaces
- Error messages or logs
- API endpoints or responses
- Web applications or interfaces

Generate a question that:
- Is specific to what's visible in the image
- Tests technical knowledge or problem-solving skills
- Is appropriate for the {} position{}
- Can be answered by someone familiar with the technology shown

EXAMPLE RESPONSES:
- If you see React code: \"What is the purpose of the useEffect hook in this component and when does it run?\"
- If you see SQL query: \"How would you optimize this query for better performance?\"
- If you see error message: \"What is causing this error and how would you debug it?\"
- If you see API endpoint: \"What HTTP method would be most appropriate for this endpoint and why?\"

Provide only the JSON response, no other text.",
            analysis_prompt,
            self.build_system_prompt(context),
            context.position.as_deref().unwrap_or("software development"),
            context.company.as_ref().map(|c| format!(" at {}", c)).unwrap_or_default()
        );
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        // Build messages with image content for vision analysis
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": analysis_prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", base64_image)
                        }
                    }
                ]
            })
        ];

        let mut payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": true,
            "private": true,
            "temperature": 0.1,  // Ultra-low temperature for maximum speed
            "max_tokens": 150,   // Much smaller for ultra-fast responses
            "top_p": 0.7,        // Very focused sampling for speed
            "presence_penalty": 0.0,
            "frequency_penalty": 0.0
        });

        // Add referrer to payload if available
        if !referrer.is_empty() {
            payload["referrer"] = serde_json::Value::String(referrer);
        }

        info!("📤 Sending vision analysis request to Pollinations...");
        
        let url = format!("{}/openai", self.base_url);
        let mut request_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0");

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
            error!("❌ Pollinations vision API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations vision API error {}: {}", status, error_text));
        }

        let response_text = response.text().await?;
        info!("✅ Received vision analysis response from Pollinations: {} characters", response_text.len());
        
        // Try to parse as OpenAI-compatible JSON response
        match serde_json::from_str::<Value>(&response_text) {
            Ok(json) => {
                // Look for content in OpenAI-compatible structure
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(first_choice) = choices.first() {
                        if let Some(message) = first_choice.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                return Ok(content.to_string());
                            }
                        }
                        // Also check for direct text field in choice
                        if let Some(text) = first_choice.get("text").and_then(|t| t.as_str()) {
                            return Ok(text.to_string());
                        }
                    }
                }
                
                // Look for direct content field
                if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                    return Ok(content.to_string());
                }
                
                // If we can't parse the expected structure, return the raw response
                Ok(response_text.trim().to_string())
            }
            Err(_) => {
                // If not valid JSON, treat as raw text response
                Ok(response_text.trim().to_string())
            }
        }
    }

    /// Analyze a screenshot using Pollinations with vision-capable models with streaming support
    pub async fn analyze_screenshot_with_vision_streaming<F>(
        &self,
        base64_image: &str,
        analysis_prompt: &str,
        context: &super::openai::InterviewContext,
        model: PollinationsModel,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        info!("🔍 Analyzing screenshot with Pollinations streaming model: {}", model.as_str());
        
        // Build a comprehensive system prompt for screenshot analysis
        let system_prompt = format!(
            "You are an expert technical interviewer analyzing a screenshot to generate relevant interview questions. {}

{}

IMPORTANT INSTRUCTIONS:
1. Carefully examine the screenshot to identify technical content
2. Generate ONE specific, relevant interview question based on what you see
3. The question should test the candidate's understanding of the visible content
4. Return your response in this EXACT JSON format:
{{
  \"generated_question\": \"Your specific interview question here?\",
  \"analysis\": \"Brief explanation of why this question is relevant\",
  \"confidence\": 0.9
}}

The screenshot might contain:
- Source code (any programming language)
- Development tools and IDEs (VS Code, IntelliJ, etc.)
- Documentation or technical articles
- System interfaces or applications
- Technical diagrams or architecture
- Database schemas or queries
- Configuration files (JSON, YAML, XML)
- Terminal/command line interfaces
- Error messages or logs
- API endpoints or responses
- Web applications or interfaces

Generate a question that:
- Is specific to what's visible in the image
- Tests technical knowledge or problem-solving skills
- Is appropriate for the {} position{}
- Can be answered by someone familiar with the technology shown

EXAMPLE RESPONSES:
- If you see React code: \"What is the purpose of the useEffect hook in this component and when does it run?\"
- If you see SQL query: \"How would you optimize this query for better performance?\"
- If you see error message: \"What is causing this error and how would you debug it?\"
- If you see API endpoint: \"What HTTP method would be most appropriate for this endpoint and why?\"

Provide only the JSON response, no other text.",
            analysis_prompt,
            self.build_system_prompt(context),
            context.position.as_deref().unwrap_or("software development"),
            context.company.as_ref().map(|c| format!(" at {}", c)).unwrap_or_default()
        );
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        // Build messages with image content for vision analysis
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": analysis_prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", base64_image)
                        }
                    }
                ]
            })
        ];

        let mut payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": true, // Enable streaming
            "private": true,
            "temperature": 0.7,
            "max_tokens": 1500
        });

        // Add referrer to payload if available
        if !referrer.is_empty() {
            payload["referrer"] = serde_json::Value::String(referrer);
        }

        info!("📤 Sending streaming vision analysis request to Pollinations...");
        
        let url = format!("{}/openai", self.base_url);
        let mut request_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "text/event-stream");

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
            error!("❌ Pollinations streaming vision API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations streaming vision API error {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);
                    
                    // Process complete lines in buffer
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer.drain(..newline_pos + 1);
                        
                        if let Some(content) = self.parse_sse_line(&line) {
                            if content == "[DONE]" {
                                info!("SSE stream completed with [DONE]");
                                return Ok(full_response);
                            }
                            
                            if !content.is_empty() {
                                // Send individual token for progressive display
                                on_token(&content);
                                full_response.push_str(&content);
                                info!("📤 POLLINATIONS VISION: Sent token to callback: '{}', token length: {}, total length: {}", 
                                    content.chars().take(50).collect::<String>(), content.len(), full_response.len());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading SSE stream: {}", e);
                    break;
                }
            }
        }
        
        // Process any remaining content in buffer
        if !buffer.trim().is_empty() {
            if let Some(content) = self.parse_sse_line(buffer.trim()) {
                if content != "[DONE]" && !content.is_empty() {
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations streaming vision API"))
        } else {
            info!("Vision streaming completed. Total response length: {}", full_response.len());
            Ok(full_response.trim().to_string())
        }
    }

    fn build_system_prompt(&self, context: &super::openai::InterviewContext) -> String {
        // Ultra-minimal prompt for maximum speed
        let mut prompt = String::from("Direct answers only. 1 sentence max.");
        
        // Only add the most essential context
        if let Some(position) = &context.position {
            if !position.is_empty() {
                prompt.push_str(&format!(" Role: {}.", position));
            }
        }
        
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
        // Try different endpoints for better reliability
        let endpoints = vec![
            "https://text.pollinations.ai/openai",
            "https://text.pollinations.ai",
        ];
        
        let mut last_error = None;
        
        for endpoint in endpoints {
            match self.try_streaming_with_endpoint(endpoint, question, context, &model, &mut on_token).await {
                Ok(result) => {
                    info!("✅ Streaming succeeded with endpoint: {}", endpoint);
                    return Ok(result);
                }
                Err(e) => {
                    error!("❌ Streaming failed with endpoint {}: {}", endpoint, e);
                    last_error = Some(e);
                }
            }
        }
        
        // If all endpoints fail, return the last error
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All streaming endpoints failed")))
    }

    async fn try_streaming_with_endpoint<F>(
        &self,
        base_url: &str,
        question: &str,
        context: &super::openai::InterviewContext,
        model: &PollinationsModel,
        on_token: &mut F,
    ) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        let system_prompt = self.build_system_prompt(context);
        let prompt = format!("{}

Q: {}

A:", system_prompt, question);

        info!("Generating streaming answer with Pollinations model: {}", model.as_str());
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        // Ultra-minimal messages for maximum speed
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": "Give direct answers fast."
            }),
            serde_json::json!({
                "role": "user",
                "content": prompt
            })
        ];

        let payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": true,
            "private": true,
            "temperature": 0.1,  // Ultra-low temperature for maximum speed
            "max_tokens": 100,   // Ultra-small for lightning-fast responses
            "top_p": 0.6,        // Very focused sampling for speed
            "presence_penalty": 0.0,
            "frequency_penalty": 0.0
        });

        let url = format!("{}/openai", self.base_url);
        info!("Pollinations streaming POST request to: {}", url);
        
        // Add referrer to payload for seed tier access
        let mut final_payload = payload;
        if !referrer.is_empty() {
            final_payload["referrer"] = serde_json::Value::String(referrer.clone());
        }

        let mut request_builder = self.client.post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Referer", referrer.as_str())  // Add referrer header for seed tier
            .json(&final_payload);

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
                    
                    // Process complete lines in buffer
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer.drain(..newline_pos + 1);
                        
                        if let Some(content) = self.parse_sse_line(&line) {
                            if content == "[DONE]" {
                                info!("SSE stream completed with [DONE]");
                                return Ok(full_response);
                            }
                            
                            if !content.is_empty() {
                                // Send individual token for progressive display
                                on_token(&content);
                                full_response.push_str(&content);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading SSE stream: {}", e);
                    break;
                }
            }
        }
        
        // Process any remaining content in buffer
        if !buffer.trim().is_empty() {
            if let Some(content) = self.parse_sse_line(buffer.trim()) {
                if content != "[DONE]" && !content.is_empty() {
                    // Send individual token for progressive display
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations streaming API"))
        } else {
            info!("Streaming completed. Total response length: {}", full_response.len());
            Ok(full_response.trim().to_string())
        }
    }

    // Helper method to parse SSE lines - optimized for better streaming performance
    fn parse_sse_line(&self, line: &str) -> Option<String> {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(":") {
            return None;
        }
        
        // Handle SSE event lines
        if line.starts_with("event: ") {
            return None; // We don't process event types, just data
        }
        
        // Handle SSE data lines
        if line.starts_with("data: ") {
            let data_content = &line[6..]; // Remove "data: " prefix
            
            // Check for completion marker
            if data_content.trim() == "[DONE]" {
                return Some("[DONE]".to_string());
            }
            
            // Try to parse as JSON for OpenAI-compatible format
            if data_content.trim().starts_with("{") {
                match serde_json::from_str::<Value>(data_content) {
                    Ok(json) => {
                        // Reduce excessive logging for better performance
                        debug!("JSON response parsed successfully");
                        
                        // Look for content in OpenAI-compatible structure first (most common)
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            if let Some(first_choice) = choices.first() {
                                if let Some(delta) = first_choice.get("delta") {
                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                        return Some(content.to_string());
                                    }
                                }
                                // Also check for direct text field in choice
                                if let Some(text) = first_choice.get("text").and_then(|t| t.as_str()) {
                                    return Some(text.to_string());
                                }
                            }
                        }
                        
                        // Look for direct content field
                        if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                            return Some(content.to_string());
                        }
                        
                        // Look for direct text field
                        if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                            return Some(text.to_string());
                        }
                        
                        // Look for message content
                        if let Some(message) = json.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                return Some(content.to_string());
                            }
                        }
                    }
                    Err(_) => {
                        // If not valid JSON, treat as raw text - reduced logging
                        debug!("Non-JSON data, treating as raw text");
                        return Some(data_content.to_string());
                    }
                }
            } else {
                // Not JSON, treat as raw text content
                return Some(data_content.to_string());
            }
        }
        
        // Handle direct text content (non-SSE format)
        else if !line.starts_with("event:") && !line.starts_with("id:") && !line.starts_with("retry:") {
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
                    
                    // Process complete lines in buffer (same logic as GET streaming)
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer.drain(..newline_pos + 1);
                        
                        if let Some(content) = self.parse_sse_line(&line) {
                            if content == "[DONE]" {
                                info!("POST SSE stream completed with [DONE]");
                                return Ok(full_response);
                            }
                            
                            if !content.is_empty() {
                                // Send individual token for progressive display
                                on_token(&content);
                                full_response.push_str(&content);
                                info!("POST streamed individual token: '{}', token length: {}, total length: {}", content.chars().take(50).collect::<String>(), content.len(), full_response.len());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading POST SSE stream: {}", e);
                    break;
                }
            }
        }
        
        // Process remaining content
        if !buffer.trim().is_empty() {
            if let Some(content) = self.parse_sse_line(buffer.trim()) {
                if content != "[DONE]" && !content.is_empty() {
                    // Send individual token for progressive display
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations POST streaming API"))
        } else {
            info!("POST streaming completed. Total response length: {}", full_response.len());
            Ok(full_response.trim().to_string())
        }
    }

    pub async fn fetch_available_models(&self) -> Result<Vec<PollinationsModel>> {
        // The Pollinations API doesn't have a /models endpoint that lists available models
        // So we'll use the known working models directly
        info!("Using predefined Pollinations models (API doesn't provide /models endpoint)");
        self.get_fallback_models()
    }
    
    fn get_fallback_models(&self) -> Result<Vec<PollinationsModel>> {
        info!("Using fallback models list (seed tier with referrer)");
        let models = vec![
            PollinationsModel::Custom("llama-fast-roblox".to_string()),
            PollinationsModel::Custom("llama-roblox".to_string()),
            PollinationsModel::Custom("openai".to_string()),
            PollinationsModel::Custom("mistral".to_string()),
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
