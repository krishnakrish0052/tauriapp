use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, error};

#[derive(Debug, Clone)]
pub enum OpenAIModel {
    GPT4Turbo,
    GPT4,
    GPT35Turbo,
}

impl OpenAIModel {
    pub fn as_str(&self) -> &str {
        match self {
            OpenAIModel::GPT4Turbo => "gpt-4-turbo-preview",
            OpenAIModel::GPT4 => "gpt-4",
            OpenAIModel::GPT35Turbo => "gpt-3.5-turbo",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gpt-4-turbo" | "GPT-4 Turbo" => Some(OpenAIModel::GPT4Turbo),
            "gpt-4" | "GPT-4" => Some(OpenAIModel::GPT4),
            "gpt-3.5-turbo" | "GPT-3.5 Turbo" => Some(OpenAIModel::GPT35Turbo),
            _ => None,
        }
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Self::from_str(s).ok_or_else(|| anyhow::anyhow!("Unknown model: {}", s))
    }
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f64,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub async fn generate_answer(
        &self,
        question: &str,
        context: &InterviewContext,
        model: OpenAIModel,
    ) -> Result<String> {
        let system_prompt = self.build_system_prompt(context);
        let user_prompt = format!("Interview Question: {}\n\nPlease provide a comprehensive and professional answer to this interview question.", question);

        let request = OpenAIRequest {
            model: model.as_str().to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            max_tokens: 1000,
            temperature: 0.7,
            stream: false,
        };

        info!("Sending request to OpenAI with model: {}", model.as_str());

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        if let Some(choice) = openai_response.choices.first() {
            info!("Received response from OpenAI");
            if let Some(usage) = openai_response.usage {
                info!(
                    "Token usage - Prompt: {}, Completion: {}, Total: {}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
            }
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI"))
        }
    }

    fn build_system_prompt(&self, context: &InterviewContext) -> String {
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

    pub async fn analyze_screen_content(
        &self,
        screen_text: &str,
        context: &InterviewContext,
        model: OpenAIModel,
    ) -> Result<String> {
        let system_prompt = format!(
            "You are analyzing screen content during an interview. Help the candidate understand what's being discussed and provide relevant talking points.\n\n{}",
            self.build_system_prompt(context)
        );

        let user_prompt = format!(
            "Screen content analysis:\n{}\n\nPlease analyze this content and provide:\n1. Key topics being discussed\n2. Relevant talking points\n3. Potential questions that might come up\n4. Suggested responses or preparation points",
            screen_text
        );

        let request = OpenAIRequest {
            model: model.as_str().to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            max_tokens: 800,
            temperature: 0.6,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        if let Some(choice) = openai_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI"))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InterviewContext {
    pub company: Option<String>,
    pub position: Option<String>,
    pub job_description: Option<String>,
}

impl InterviewContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_company(mut self, company: String) -> Self {
        self.company = Some(company);
        self
    }

    pub fn with_position(mut self, position: String) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_job_description(mut self, job_description: String) -> Self {
        self.job_description = Some(job_description);
        self
    }
}
