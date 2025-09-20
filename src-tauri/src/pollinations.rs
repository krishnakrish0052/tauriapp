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
                    // Official Pollinations models from API (2025)
                    "deepseek-reasoning" => "DeepSeek R1 0528 (Bedrock)",
                    "gemini" => "Gemini 2.5 Flash Lite (api.navy)",
                    "mistral" => "Mistral Small 3.1 24B",
                    "nova-fast" => "Amazon Nova Micro (Bedrock)",
                    "openai" => "OpenAI GPT-5 Nano",
                    "openai-audio" => "OpenAI GPT-4o Mini Audio Preview",
                    "openai-fast" => "OpenAI GPT-4.1 Nano",
                    "openai-reasoning" => "OpenAI o4-mini (api.navy)",
                    "qwen-coder" => "Qwen 2.5 Coder 32B",
                    "roblox-rp" => "Llama 3.1 8B Instruct (Cross-Region Bedrock)",
                    "bidara" => "BIDARA (Biomimetic Designer and Research Assistant by NASA)",
                    "evil" => "Evil (Uncensored)",
                    "midijourney" => "MIDIjourney",
                    "mirexa" => "Mirexa AI Companion",
                    "rtist" => "Rtist",
                    "unity" => "Unity Unrestricted Agent",
                    // Legacy compatibility
                    "llama-fast-roblox" => "Llama 3.1 8B Instruct (Legacy)",
                    "llama-roblox" => "Llama 3.1 8B Instruct (Legacy)",
                    _ => s.as_str(),
                }
            }
        }
    }
    
    /// Check if this model supports vision capabilities
    pub fn supports_vision(&self) -> bool {
        match self.as_str() {
            "gemini" | "openai" | "openai-fast" | "openai-reasoning" | "openai-audio" |
            "bidara" | "evil" | "mirexa" | "unity" => true,
            _ => false,
        }
    }
    
    /// Check if this model supports audio input/output
    pub fn supports_audio(&self) -> bool {
        matches!(self.as_str(), "openai-audio")
    }
    
    /// Get the tier required for this model
    pub fn required_tier(&self) -> &str {
        match self.as_str() {
            "deepseek-reasoning" | "openai-audio" | "openai-reasoning" | "roblox-rp" |
            "evil" | "mirexa" | "rtist" | "unity" => "seed",
            _ => "anonymous",
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
    /// Quick health check to determine if Pollinations service is available
    pub async fn health_check(&self) -> bool {
        info!("🏥 Performing Pollinations health check...");
        
        // Use a simple GET request to test connectivity
        let health_check_url = "https://text.pollinations.ai/models";
        
        match tokio::time::timeout(
            std::time::Duration::from_secs(5), // Very short timeout for health check
            self.client.get(health_check_url)
                .header("User-Agent", "MockMate/1.0")
                .send()
        ).await {
            Ok(Ok(response)) => {
                let is_healthy = response.status().is_success() || response.status().as_u16() < 500;
                info!("🏥 Pollinations health check result: {} (status: {})", 
                      if is_healthy { "✅ HEALTHY" } else { "❌ UNHEALTHY" }, 
                      response.status());
                is_healthy
            },
            Ok(Err(e)) => {
                warn!("🏥 Pollinations health check failed: {}", e);
                false
            },
            Err(_) => {
                warn!("🏥 Pollinations health check timed out");
                false
            }
        }
    }
    
    /// Check if an error is a temporary infrastructure issue
    fn is_temporary_infrastructure_error(error_text: &str, status_code: u16) -> bool {
        status_code == 530 || 
        error_text.contains("Cloudflare Tunnel error") ||
        error_text.contains("Infrastructure issue") ||
        error_text.contains("temporarily unavailable")
    }
    
    pub fn new(api_key: String, referrer: String) -> Self {
        // Optimized HTTP client configuration for fast failure on infrastructure issues
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))     // Shorter timeout for faster failure
            .connect_timeout(std::time::Duration::from_secs(3))  // Faster connection timeout
            .tcp_keepalive(std::time::Duration::from_secs(15))
            .pool_idle_timeout(std::time::Duration::from_secs(5))
            .pool_max_idle_per_host(10)  // Reduced for faster failure detection
            .http2_keep_alive_interval(std::time::Duration::from_secs(5))
            .user_agent("MockMate/1.0")  // Set default user agent
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
        
        // Quick health check first to fail fast if service is down
        if !self.health_check().await {
            let error_msg = "❌ Pollinations service is currently unavailable (health check failed). This may be due to temporary infrastructure issues. Please try using OpenAI or wait a few minutes and retry.";
            error!("Health check failed - Pollinations service unavailable");
            return Err(anyhow::anyhow!(error_msg));
        }
        
        // Try different endpoints and formats for Pollinations API
        let endpoints = vec![
            ("https://text.pollinations.ai/openai", "json"),
            ("https://text.pollinations.ai", "text"),
        ];
        
        let mut last_error_details = String::new();
        let mut is_infrastructure_issue = false;
        
        for (base_url, response_format) in endpoints {
            info!("Trying Pollinations endpoint: {} (format: {})", base_url, response_format);
            
            match self.try_generate_with_endpoint(base_url, &prompt, &model, response_format).await {
                Ok(result) => {
                    info!("✅ Successfully generated answer with endpoint: {}", base_url);
                    return Ok(result);
                }
                Err(e) => {
                    let error_str = e.to_string();
                    error!("❌ Failed with endpoint {}: {}", base_url, error_str);
                    
                    // Check if this is an infrastructure issue
                    if error_str.contains("HTTP 530") || Self::is_temporary_infrastructure_error(&error_str, 530) {
                        is_infrastructure_issue = true;
                    }
                    
                    last_error_details = error_str;
                    continue;
                }
            }
        }
        
        // Provide specific error messages based on the type of failure
        let error_msg = if is_infrastructure_issue {
            "❌ Pollinations service is experiencing infrastructure issues (HTTP 530 - Cloudflare Tunnel errors). This is temporary and should resolve within a few minutes. Please try using OpenAI instead, or wait and retry.".to_string()
        } else {
            format!("❌ All Pollinations API endpoints failed. Last error: {}. Please try using OpenAI or check your network connection.", last_error_details)
        };
        
        error!("❌ All Pollinations endpoints failed - {}", if is_infrastructure_issue { "infrastructure issue" } else { "other error" });
        Err(anyhow::anyhow!(error_msg))
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
            .append_pair("temperature", "0.3")  // Balanced temperature
            .append_pair("max_tokens", "150");  // Better token limit

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

    /// Answer questions found within a screenshot using vision-capable models (streaming)
    pub async fn answer_screenshot_questions_streaming<F>(
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
        info!("🖼️ Answering on-screen questions with Pollinations streaming model: {}", model.as_str());
        
        // Build a focused system prompt for answering questions from screenshot/chat
        let system_prompt = format!(
            "You are an elite interview copilot. Carefully read the screenshot content (UI, chat, slides, docs).{}

{}

Instructions:
- Identify any text fragments that look like questions (especially from chat or prompts).
- Provide the most accurate, concise answers directly. If multiple questions are present, answer each on a new line prefixed with '-'.
- Prefer practical, interview-ready responses (30–60 seconds when spoken).
- If no clear question is present, extract the most relevant technical topic and provide a brief summary.
- Do not include extra commentary; just the answers.",
            if let Some(company) = &context.company { format!(" Company: {}.", company) } else { String::new() },
            self.build_system_prompt(context)
        );
        
        let api_key = std::env::var("POLLINATIONS_API_KEY").unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER").unwrap_or_else(|_| "mockmate".to_string());

        // Messages with image input per OpenAI-compatible format
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            }),
            serde_json::json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": analysis_prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", base64_image)}}
                ]
            })
        ];

        let mut payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": true,
            "private": true,
            "temperature": 0.1,
            "max_tokens": 400
        });

        if !referrer.is_empty() {
            payload["referrer"] = serde_json::Value::String(referrer.clone());
        }

        let url = format!("{}/openai", self.base_url);
        let mut request_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "text/event-stream")
            .header("Referer", referrer);

        if !api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder.json(&payload).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Pollinations screenshot-answer API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations screenshot-answer API error {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer.drain(..newline_pos + 1);
                        if let Some(content) = self.parse_sse_line(&line) {
                            if content == "[DONE]" { return Ok(full_response); }
                            if !content.is_empty() {
                                on_token(&content);
                                full_response.push_str(&content);
                            }
                        }
                    }
                }
                Err(e) => { error!("Error reading SSE stream: {}", e); break; }
            }
        }

        if !buffer.trim().is_empty() {
            if let Some(content) = self.parse_sse_line(buffer.trim()) {
                if content != "[DONE]" && !content.is_empty() {
                    on_token(&content);
                    full_response.push_str(&content);
                }
            }
        }

        if full_response.trim().is_empty() {
            Err(anyhow::anyhow!("Empty response from Pollinations screenshot-answer API"))
        } else {
            Ok(full_response.trim().to_string())
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
        // Optimized prompt for speed and quality balance
        let mut prompt = String::from("Provide clear, concise answers. Be direct and helpful.");
        
        // Add essential context efficiently
        if let Some(position) = &context.position {
            if !position.is_empty() {
                prompt.push_str(&format!(" Context: {}.", position));
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
        info!("🚀 Starting Pollinations streaming for model: {}", model.as_str());
        
        // Quick health check first to fail fast if service is down
        if !self.health_check().await {
            let error_msg = "❌ Pollinations service is currently unavailable (health check failed). Streaming cannot proceed.";
            error!("Health check failed - Pollinations streaming unavailable");
            return Err(anyhow::anyhow!(error_msg));
        }
        
        // Try different endpoints for better reliability, fastest first
        let endpoints = vec![
            "https://text.pollinations.ai",         // Direct endpoint (often faster)
            "https://text.pollinations.ai/openai",  // OpenAI-compatible endpoint
        ];
        
        let mut last_error = None;
        let mut is_infrastructure_issue = false;
        
        for endpoint in endpoints {
            match self.try_streaming_with_endpoint(endpoint, question, context, &model, &mut on_token).await {
                Ok(result) => {
                    info!("✅ Streaming succeeded with endpoint: {}", endpoint);
                    return Ok(result);
                }
                Err(e) => {
                    let error_str = e.to_string();
                    error!("❌ Streaming failed with endpoint {}: {}", endpoint, error_str);
                    
                    // Check if this is an infrastructure issue
                    if error_str.contains("HTTP 530") || Self::is_temporary_infrastructure_error(&error_str, 530) {
                        is_infrastructure_issue = true;
                    }
                    
                    last_error = Some(e);
                }
            }
        }
        
        // Provide specific error messages based on the type of failure
        let final_error = if is_infrastructure_issue {
            anyhow::anyhow!("❌ Pollinations streaming failed due to infrastructure issues (HTTP 530 - Cloudflare Tunnel errors). This is temporary. Please try using OpenAI or wait and retry.")
        } else {
            last_error.unwrap_or_else(|| anyhow::anyhow!("All streaming endpoints failed"))
        };
        
        Err(final_error)
    }

    async fn try_streaming_with_endpoint<F>(
        &self,
        _base_url: &str,
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

        info!("🚀 Generating streaming answer with Pollinations model: {} (optimized for speed)", model.as_str());
        let start_time = std::time::Instant::now();
        
        // Get API key and referrer from environment
        let api_key = std::env::var("POLLINATIONS_API_KEY")
            .unwrap_or_default();
        let referrer = std::env::var("POLLINATIONS_REFERER")
            .unwrap_or_else(|_| "mockmate".to_string());

        // Optimized messages for speed and quality
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": "You are a helpful AI assistant. Respond quickly and clearly."
            }),
            serde_json::json!({
                "role": "user",
                "content": prompt
            })
        ];

        // Optimize parameters based on model type for maximum speed
        let (temperature, max_tokens, top_p) = match model.as_str() {
            // Ultra-fast models (anonymous tier)
            "nova-fast" => (0.1, 120, 0.7),          // Amazon Nova Micro (ultra-fast)
            "openai-fast" => (0.1, 130, 0.75),       // OpenAI GPT-4.1 Nano (fast)
            "gemini" => (0.2, 150, 0.8),             // Gemini 2.5 Flash Lite (vision)
            "qwen-coder" => (0.2, 160, 0.8),         // Qwen Coder (specialized)
            // Balanced models
            "mistral" => (0.3, 200, 0.9),            // Mistral Small 3.1 24B
            "openai" => (0.3, 250, 0.9),             // OpenAI GPT-5 Nano (vision)
            "bidara" => (0.3, 220, 0.85),            // NASA BIDARA (vision)
            "midijourney" => (0.4, 180, 0.9),        // MIDIjourney
            // Premium models (seed tier)
            "deepseek-reasoning" => (0.1, 300, 0.8), // DeepSeek R1 (reasoning)
            "openai-reasoning" => (0.1, 280, 0.8),   // OpenAI o4-mini (reasoning + vision)
            "openai-audio" => (0.3, 250, 0.9),       // Audio + vision model
            "roblox-rp" => (0.3, 200, 0.85),         // Llama 3.1 8B
            "mirexa" => (0.4, 230, 0.9),             // Mirexa AI Companion (vision)
            "rtist" => (0.4, 200, 0.9),              // Rtist
            // Uncensored models
            "evil" => (0.4, 220, 0.9),               // Evil (uncensored + vision)
            "unity" => (0.3, 240, 0.9),              // Unity Unrestricted (vision)
            // Legacy compatibility
            "llama-fast-roblox" | "llama-roblox" => (0.3, 180, 0.85),
            _ => (0.3, 200, 0.9),                     // Default balanced settings
        };
        
        let mut payload = serde_json::json!({
            "model": model.as_str(),
            "messages": messages,
            "stream": true,
            "private": true,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "top_p": top_p,
            "presence_penalty": 0.0,
            "frequency_penalty": 0.0
        });
        
        // Add speed optimizations for fast models
        let is_fast_model = matches!(model.as_str(), 
            "nova-fast" | "gemini" | "deepseek-reasoning") || 
            model.as_str().contains("fast");
            
        if is_fast_model {
            payload["seed"] = serde_json::Value::Number(serde_json::Number::from(42));
            payload["best_of"] = serde_json::Value::Number(serde_json::Number::from(1));
            // Additional speed optimization
            payload["stream"] = serde_json::Value::Bool(true);
        }

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
            .header("Connection", "keep-alive")
            .header("X-Requested-With", "MockMate")
            .header("Referer", referrer.as_str())  // Add referrer header for seed tier
            .json(&final_payload);

        // Add Bearer token if available
        if !api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder.send().await?;
        let request_time = start_time.elapsed();
        info!("📡 Request sent to Pollinations API in {:?}", request_time);

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Pollinations streaming API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Pollinations streaming API error {}: {}", status, error_text));
        }
        
        info!("✅ Pollinations API responded with status: {}", response.status());

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();
        let mut first_token_time: Option<std::time::Instant> = None;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);
                    
                    // Process complete lines in buffer with immediate flushing
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer.drain(..newline_pos + 1);
                        
                        if let Some(content) = self.parse_sse_line(&line) {
                            if content == "[DONE]" {
                                debug!("SSE stream completed with [DONE]");
                                return Ok(full_response);
                            }
                            
                            if !content.is_empty() {
                                // Track first token timing
                                if first_token_time.is_none() {
                                    first_token_time = Some(std::time::Instant::now());
                                    let time_to_first_token = start_time.elapsed();
                                    info!("⚡ First token received in {:?}", time_to_first_token);
                                }
                                
                                // Send individual token immediately for progressive display
                                // Optimize for speed: reduce debug logging for better performance
                                if content.len() <= 50 {
                                    debug!("📤 Token: '{}' ({})", content.chars().take(20).collect::<String>(), content.len());
                                } else {
                                    debug!("📤 Token: {}chars", content.len());
                                }
                                on_token(&content);
                                full_response.push_str(&content);
                            }
                        }
                    }
                    
                    // Also try to process partial lines for immediate response
                    if !buffer.is_empty() && !buffer.contains('\n') {
                        // Check if buffer contains a complete token without newline
                        if let Some(content) = self.try_parse_partial_sse(&buffer) {
                            if !content.is_empty() {
                                on_token(&content);
                                full_response.push_str(&content);
                                buffer.clear();
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

        let total_time = start_time.elapsed();
        if full_response.trim().is_empty() {
            error!("❌ Empty response from Pollinations streaming API after {:?}", total_time);
            Err(anyhow::anyhow!("Empty response from Pollinations streaming API"))
        } else {
            info!("✅ Streaming completed in {:?}. Total response length: {} characters", total_time, full_response.len());
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
        info!("Fetching available models from Pollinations API...");
        
        // Use the official models API endpoint
        match self.fetch_models_from_official_api().await {
            Ok(models) if !models.is_empty() => {
                info!("Successfully fetched {} models from official Pollinations API", models.len());
                Ok(models)
            }
            Ok(_) => {
                info!("Official API returned empty models list, using known working models");
                self.get_fallback_models()
            }
            Err(e) => {
                warn!("Failed to fetch from official API ({}), using known working models", e);
                self.get_fallback_models()
            }
        }
    }
    
    async fn fetch_models_from_official_api(&self) -> Result<Vec<PollinationsModel>> {
        let endpoint = "https://text.pollinations.ai/models";
        
        info!("Fetching models from official endpoint: {}", endpoint);
        
        let response = self.client.get(endpoint)
            .header("User-Agent", "MockMate/1.0")
            .header("Accept", "application/json")
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API returned status: {}", response.status()));
        }
        
        let text = response.text().await?;
        info!("Received models data: {} characters", text.len());
        
        // Parse the models array from the official API
        match serde_json::from_str::<Vec<serde_json::Value>>(&text) {
            Ok(models_array) => {
                let mut models = Vec::new();
                for model in models_array {
                    if let Some(name) = model.get("name").and_then(|v| v.as_str()) {
                        models.push(PollinationsModel::Custom(name.to_string()));
                    }
                }
                
                if models.is_empty() {
                    warn!("No models found in API response");
                    return Err(anyhow::anyhow!("No models found in API response"));
                }
                
                info!("Parsed {} models from official API", models.len());
                Ok(models)
            }
            Err(e) => {
                error!("Failed to parse models JSON: {}", e);
                Err(anyhow::anyhow!("Failed to parse models JSON: {}", e))
            }
        }
    }
    
    fn parse_models_from_json(&self, json: &serde_json::Value) -> Option<Vec<PollinationsModel>> {
        // Try different JSON structures that might contain model information
        if let Some(models_array) = json.get("data").and_then(|v| v.as_array()) {
            let mut models = Vec::new();
            for model in models_array {
                if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
                    models.push(PollinationsModel::Custom(id.to_string()));
                }
            }
            if !models.is_empty() {
                return Some(models);
            }
        }
        
        if let Some(models_array) = json.as_array() {
            let mut models = Vec::new();
            for model in models_array {
                if let Some(id) = model.as_str() {
                    models.push(PollinationsModel::Custom(id.to_string()));
                }
            }
            if !models.is_empty() {
                return Some(models);
            }
        }
        
        None
    }
    
    // Helper method to try parsing partial SSE data for immediate response
    fn try_parse_partial_sse(&self, buffer: &str) -> Option<String> {
        let buffer = buffer.trim();
        
        // Skip if empty or not data line
        if buffer.is_empty() || !buffer.starts_with("data: ") {
            return None;
        }
        
        let data_content = &buffer[6..]; // Remove "data: " prefix
        
        // Skip completion markers
        if data_content.trim() == "[DONE]" {
            return None;
        }
        
        // Try to extract content from partial JSON
        if data_content.trim().starts_with("{") {
            // Attempt to parse even partial JSON for immediate response
            if let Ok(json) = serde_json::from_str::<Value>(data_content) {
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(first_choice) = choices.first() {
                        if let Some(delta) = first_choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                return Some(content.to_string());
                            }
                        }
                        if let Some(text) = first_choice.get("text").and_then(|t| t.as_str()) {
                            return Some(text.to_string());
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn get_fallback_models(&self) -> Result<Vec<PollinationsModel>> {
        info!("Using fallback models list based on official Pollinations API");
        let models = vec![
            // Fast models first for better performance (anonymous tier)
            PollinationsModel::Custom("nova-fast".to_string()),     // Amazon Nova Micro (Bedrock)
            PollinationsModel::Custom("gemini".to_string()),        // Gemini 2.5 Flash Lite (Vision)
            PollinationsModel::Custom("mistral".to_string()),       // Mistral Small 3.1 24B
            PollinationsModel::Custom("openai".to_string()),        // OpenAI GPT-5 Nano (Vision)
            PollinationsModel::Custom("openai-fast".to_string()),   // OpenAI GPT-4.1 Nano (Vision)
            PollinationsModel::Custom("qwen-coder".to_string()),    // Qwen 2.5 Coder 32B
            PollinationsModel::Custom("bidara".to_string()),        // NASA BIDARA (Vision)
            PollinationsModel::Custom("midijourney".to_string()),   // MIDIjourney
            // Seed tier models (higher quality)
            PollinationsModel::Custom("deepseek-reasoning".to_string()), // DeepSeek R1 0528 (Reasoning)
            PollinationsModel::Custom("openai-audio".to_string()),  // OpenAI GPT-4o Mini Audio (Vision + Audio)
            PollinationsModel::Custom("openai-reasoning".to_string()), // OpenAI o4-mini (Vision + Reasoning)
            PollinationsModel::Custom("roblox-rp".to_string()),     // Llama 3.1 8B Instruct
            PollinationsModel::Custom("mirexa".to_string()),        // Mirexa AI Companion (Vision)
            PollinationsModel::Custom("rtist".to_string()),         // Rtist
            PollinationsModel::Custom("evil".to_string()),          // Evil (Uncensored, Vision)
            PollinationsModel::Custom("unity".to_string()),         // Unity Unrestricted Agent (Vision)
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
