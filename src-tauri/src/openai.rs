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
    content: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ImageContent {
    r#type: String,
    image_url: ImageUrl,
}

#[derive(Serialize, Deserialize)]
struct ImageUrl {
    url: String,
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
                    content: serde_json::Value::String(system_prompt),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: serde_json::Value::String(user_prompt),
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
            // Extract content as string
            let content = match &choice.message.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI"))
        }
    }

    fn build_system_prompt(&self, context: &InterviewContext) -> String {
        let mut prompt = String::new();
        
        // Core role and personality
        prompt.push_str("You are an expert interview coach and career advisor. Your role is to help the candidate succeed by providing thoughtful, authentic, and compelling answers that showcase their qualifications naturally.");
        
        // Personalized greeting if user name is available
        if let Some(user_name) = &context.user_name {
            prompt.push_str(&format!("\n\nYou are specifically assisting {}.", user_name.trim()));
        }
        
        // Interview context section
        prompt.push_str("\n\n=== INTERVIEW CONTEXT ===");
        
        if let Some(company) = &context.company {
            prompt.push_str(&format!("\nTarget Company: {}", company));
            prompt.push_str(&format!("\n• Tailor your responses to align with {}'s values, culture, and industry reputation", company));
            prompt.push_str(&format!("\n• Reference {}'s known projects, initiatives, or business model when relevant", company));
        }
        
        if let Some(position) = &context.position {
            prompt.push_str(&format!("\nRole: {}", position));
            prompt.push_str(&format!("\n• Focus on skills and experiences directly relevant to {} responsibilities", position));
            prompt.push_str(&format!("\n• Demonstrate understanding of what success looks like in this {} role", position));
        }
        
        // Difficulty and experience level customization
        if let Some(difficulty) = &context.difficulty_level {
            match difficulty.to_lowercase().as_str() {
                "entry" | "junior" | "beginner" => {
                    prompt.push_str("\nExperience Level: Entry-Level/Junior");
                    prompt.push_str("\n• Emphasize learning agility, academic projects, internships, and personal projects");
                    prompt.push_str("\n• Show enthusiasm and coachability rather than extensive experience");
                    prompt.push_str("\n• Highlight transferable skills and potential for growth");
                }
                "mid" | "intermediate" | "medium" => {
                    prompt.push_str("\nExperience Level: Mid-Level");
                    prompt.push_str("\n• Balance proven experience with continued learning and growth mindset");
                    prompt.push_str("\n• Demonstrate leadership potential and cross-functional collaboration");
                    prompt.push_str("\n• Show progression in responsibilities and impact");
                }
                "senior" | "advanced" | "high" => {
                    prompt.push_str("\nExperience Level: Senior-Level");
                    prompt.push_str("\n• Emphasize strategic thinking, team leadership, and business impact");
                    prompt.push_str("\n• Demonstrate ability to mentor others and drive organizational change");
                    prompt.push_str("\n• Focus on scalable solutions and long-term vision");
                }
                _ => {
                    prompt.push_str(&format!("\nDifficulty Level: {}", difficulty));
                }
            }
        }
        
        if let Some(session_type) = &context.session_type {
            prompt.push_str(&format!("\nInterview Type: {}", session_type));
            match session_type.to_lowercase().as_str() {
                "behavioral" => {
                    prompt.push_str("\n• Use STAR method (Situation, Task, Action, Result) for storytelling");
                    prompt.push_str("\n• Focus on specific examples that demonstrate soft skills and cultural fit");
                }
                "technical" => {
                    prompt.push_str("\n• Provide clear, logical explanations with step-by-step reasoning");
                    prompt.push_str("\n• Consider trade-offs, edge cases, and scalability when relevant");
                }
                "case" | "case study" => {
                    prompt.push_str("\n• Structure responses with clear problem-solving frameworks");
                    prompt.push_str("\n• Ask clarifying questions and state assumptions explicitly");
                }
                _ => {}
            }
        }
        
        if let Some(job_description) = &context.job_description {
            if !job_description.is_empty() {
                let job_desc_summary = if job_description.len() > 300 {
                    format!("{}...", &job_description[..300])
                } else {
                    job_description.clone()
                };
                prompt.push_str(&format!("\n\nJob Description Summary: {}", job_desc_summary));
                prompt.push_str("\n• Align your responses with the specific requirements and qualifications mentioned");
            }
        }
        
        // Resume context if available
        if let Some(resume) = &context.resume_content {
            if !resume.is_empty() {
                prompt.push_str("\n\n=== CANDIDATE BACKGROUND ===\n");
                let resume_summary = if resume.len() > 500 {
                    format!("{}...", &resume[..500])
                } else {
                    resume.clone()
                };
                prompt.push_str(&format!("Resume Summary: {}", resume_summary));
                prompt.push_str("\n• Draw from this background to provide authentic, personalized responses");
                prompt.push_str("\n• Reference specific experiences, skills, and achievements when relevant");
            }
        }
        
        // Response style guidelines
        prompt.push_str("\n\n=== RESPONSE GUIDELINES ===");
        prompt.push_str("\n\n🎯 AUTHENTICITY & TONE:");
        prompt.push_str("\n• Write as if you're the candidate speaking naturally and confidently");
        prompt.push_str("\n• Use first person (\"I\", \"my\", \"we\") to make responses personal and engaging");
        prompt.push_str("\n• Match the energy and professionalism appropriate for the role and company");
        prompt.push_str("\n• Avoid robotic or templated language - sound human and genuine");
        
        prompt.push_str("\n\n📝 STRUCTURE & CONTENT:");
        prompt.push_str("\n• Lead with confidence: start with a clear, direct response to the question");
        prompt.push_str("\n• Support with specifics: provide concrete examples, metrics, or scenarios");
        prompt.push_str("\n• Connect to value: explicitly link your response to how you'd contribute to their team");
        prompt.push_str("\n• Keep it conversational: aim for 30-90 seconds when spoken aloud");
        
        prompt.push_str("\n\n🚀 STRATEGIC APPROACH:");
        prompt.push_str("\n• Turn every question into an opportunity to demonstrate value and fit");
        prompt.push_str("\n• Show don't just tell: use specific stories and examples to illustrate points");
        prompt.push_str("\n• Address potential concerns proactively while staying positive");
        prompt.push_str("\n• End responses with forward momentum or a thoughtful question when appropriate");
        
        // Interview type specific guidance
        prompt.push_str("\n\n💡 QUESTION TYPE ADAPTATIONS:");
        prompt.push_str("\n• Behavioral: Use STAR method but make it conversational, not mechanical");
        prompt.push_str("\n• Technical: Explain your thinking process, consider alternatives, show expertise depth");
        prompt.push_str("\n• Hypothetical: Think out loud, ask clarifying questions, show problem-solving approach");
        prompt.push_str("\n• Culture/Fit: Be authentic about values while showing genuine enthusiasm for their mission");
        
        prompt.push_str("\n\nRemember: Your goal is to help the candidate sound competent, confident, and genuinely excited about the opportunity while being completely authentic to who they are as a professional.");
        
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
                    content: serde_json::Value::String(system_prompt),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: serde_json::Value::String(user_prompt),
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
            // Extract content as string
            let content = match &choice.message.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI"))
        }
    }

    /// Analyze a screenshot using OpenAI's vision models
    pub async fn analyze_screenshot_with_vision(
        &self,
        base64_image: &str,
        analysis_prompt: &str,
        context: &InterviewContext,
        model: OpenAIModel,
    ) -> Result<String> {
        info!("🔍 Analyzing screenshot with OpenAI Vision API...");
        
        // For vision analysis, we need to use a vision-capable model
        let vision_model = match model {
            OpenAIModel::GPT4Turbo => "gpt-4-vision-preview",
            OpenAIModel::GPT4 => "gpt-4-vision-preview",
            _ => {
                // Fallback to text-based analysis for non-vision models
                return self.analyze_screen_content(
                    "Unable to analyze screenshot directly. Please describe the screen content.", 
                    context, 
                    model
                ).await;
            }
        };
        
        let system_prompt = format!(
            "You are an expert technical interviewer analyzing a screenshot. {}\n\n{}",
            analysis_prompt,
            self.build_system_prompt(context)
        );
        
        // Build the vision message with image data
        let image_content = serde_json::json!([
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
        ]);
        
        let request = OpenAIRequest {
            model: vision_model.to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: serde_json::Value::String(system_prompt),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: image_content,
                },
            ],
            max_tokens: 1500,
            temperature: 0.7,
            stream: false,
        };
        
        info!("📤 Sending vision analysis request to OpenAI...");
        
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
            error!("❌ OpenAI Vision API error: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI Vision API error: {}", error_text));
        }
        
        let openai_response: OpenAIResponse = response.json().await?;
        
        if let Some(choice) = openai_response.choices.first() {
            info!("✅ Received vision analysis response from OpenAI");
            if let Some(usage) = openai_response.usage {
                info!(
                    "Vision Token usage - Prompt: {}, Completion: {}, Total: {}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
            }
            
            // Extract content as string
            let content = match &choice.message.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No response choices from OpenAI Vision"))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InterviewContext {
    pub company: Option<String>,
    pub position: Option<String>,
    pub job_description: Option<String>,
    // Enhanced user context fields
    pub user_name: Option<String>,
    pub difficulty_level: Option<String>,
    pub session_type: Option<String>,
    pub resume_content: Option<String>,
    pub user_experience_level: Option<String>,
    pub interview_style: Option<String>,
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
    
    pub fn with_user_name(mut self, user_name: String) -> Self {
        self.user_name = Some(user_name);
        self
    }
    
    pub fn with_difficulty_level(mut self, difficulty_level: String) -> Self {
        self.difficulty_level = Some(difficulty_level);
        self
    }
    
    pub fn with_session_type(mut self, session_type: String) -> Self {
        self.session_type = Some(session_type);
        self
    }
    
    pub fn with_resume_content(mut self, resume_content: String) -> Self {
        self.resume_content = Some(resume_content);
        self
    }
    
    pub fn with_experience_level(mut self, experience_level: String) -> Self {
        self.user_experience_level = Some(experience_level);
        self
    }
    
    pub fn with_interview_style(mut self, interview_style: String) -> Self {
        self.interview_style = Some(interview_style);
        self
    }
}
