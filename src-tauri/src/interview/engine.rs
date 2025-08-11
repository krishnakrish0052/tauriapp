use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use reqwest::Client;
use crate::database::DatabaseManager;
use super::{Question, Answer, AIFeedback};

#[derive(Serialize, Deserialize)]
pub struct InterviewEngine {
    pub session_id: String,
    pub job_title: String,
    pub job_description: String,
    pub difficulty: String,
    pub resume_content: Option<String>,
    pub current_question_number: u32,
    pub openai_api_key: Option<String>,
}

impl InterviewEngine {
    pub async fn new(session_id: String, config: crate::session::InterviewConfig) -> Result<Self, String> {
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        
        if openai_api_key.is_none() {
            warn!("OpenAI API key not found in environment - AI features will be limited");
        }
        
        Ok(InterviewEngine {
            session_id,
            job_title: config.job_title,
            job_description: config.job_description,
            difficulty: config.difficulty,
            resume_content: config.resume_content,
            current_question_number: 0,
            openai_api_key,
        })
    }
    
    pub async fn generate_next_question(&mut self) -> Result<Question, String> {
        self.current_question_number += 1;
        
        info!("🤖 Generating question #{} for {} position", self.current_question_number, self.job_title);
        
        let context_prompt = format!(
            "Generate interview question #{} for a {} position ({} level).
            Job Description: {}
            {}

            Generate a relevant, challenging question that tests both technical skills and problem-solving ability.
            Make the question specific to the role and difficulty level.
            Return only the question text, no additional formatting.",
            self.current_question_number,
            self.job_title,
            self.difficulty,
            self.job_description,
            self.resume_content
                .as_ref()
                .map(|r| format!("Candidate Resume: {}", r))
                .unwrap_or_default()
        );
        
        let question_text = if let Some(api_key) = &self.openai_api_key {
            self.call_openai_api(&context_prompt, api_key).await?
        } else {
            // Fallback to sample questions if no API key
            self.generate_sample_question()
        };
        
        let question = Question {
            id: Uuid::new_v4().to_string(),
            number: self.current_question_number,
            text: question_text,
            category: self.determine_question_category(),
            difficulty_level: self.difficulty.clone(),
            expected_duration: self.calculate_expected_duration(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // Sync question to main database
        self.sync_question_to_db(&question).await?;
        
        info!("✅ Generated and stored question: {}", question.text.chars().take(50).collect::<String>());
        
        Ok(question)
    }
    
    pub async fn evaluate_answer(&self, question: &Question, answer: &Answer) -> Result<AIFeedback, String> {
        info!("🧠 Evaluating answer for question #{}", question.number);
        
        if let Some(api_key) = &self.openai_api_key {
            self.evaluate_with_ai(question, answer, api_key).await
        } else {
            // Fallback to basic evaluation
            Ok(self.generate_sample_feedback(answer))
        }
    }
    
    async fn call_openai_api(&self, prompt: &str, api_key: &str) -> Result<String, String> {
        let client = Client::new();
        
        let request_body = serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 200,
            "temperature": 0.7
        });
        
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("OpenAI API error: {}", response.status()));
        }
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in OpenAI response")?
            .trim()
            .to_string();
        
        Ok(content)
    }
    
    async fn evaluate_with_ai(&self, question: &Question, answer: &Answer, api_key: &str) -> Result<AIFeedback, String> {
        let evaluation_prompt = format!(
            "Evaluate this interview answer for a {} position ({} level):

            Question: {}
            Answer: {}
            Response Time: {} seconds
            
            Provide detailed feedback with:
            1. Score (1-10)
            2. Key strengths (max 3 points)
            3. Areas for improvement (max 3 points)
            4. Specific suggestions (max 3 points)
            
            Return as JSON with fields: score, strengths, improvements, detailed_feedback, follow_up_suggestions",
            self.job_title,
            self.difficulty,
            question.text,
            answer.text,
            answer.response_time
        );
        
        let feedback_response = self.call_openai_api(&evaluation_prompt, api_key).await?;
        
        // Try to parse as JSON, fallback to structured text if needed
        match serde_json::from_str::<serde_json::Value>(&feedback_response) {
            Ok(json) => {
                let feedback = AIFeedback {
                    score: json["score"].as_u64().unwrap_or(5) as u8,
                    strengths: json["strengths"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_else(|| vec!["Clear communication".to_string()]),
                    improvements: json["improvements"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_else(|| vec!["More specific examples needed".to_string()]),
                    detailed_feedback: json["detailed_feedback"].as_str().unwrap_or(&feedback_response).to_string(),
                    follow_up_suggestions: json["follow_up_suggestions"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_else(|| vec!["Practice with more examples".to_string()]),
                };
                
                // Sync answer and feedback to main database
                self.sync_answer_to_db(answer, &feedback).await?;
                
                Ok(feedback)
            }
            Err(_) => {
                // Fallback to parsing the text response
                let feedback = AIFeedback {
                    score: 6, // Default score
                    strengths: vec!["Response provided".to_string()],
                    improvements: vec!["More detail could be helpful".to_string()],
                    detailed_feedback: feedback_response,
                    follow_up_suggestions: vec!["Practice similar questions".to_string()],
                };
                
                self.sync_answer_to_db(answer, &feedback).await?;
                
                Ok(feedback)
            }
        }
    }
    
    fn generate_sample_question(&self) -> String {
        let sample_questions = vec![
            "Tell me about a challenging project you worked on recently.",
            "How do you handle tight deadlines and pressure?",
            "Describe your experience with team collaboration.",
            "What interests you most about this role?",
            "How do you stay updated with industry trends?",
        ];
        
        let index = (self.current_question_number as usize - 1) % sample_questions.len();
        sample_questions[index].to_string()
    }
    
    fn generate_sample_feedback(&self, answer: &Answer) -> AIFeedback {
        AIFeedback {
            score: if answer.response_time < 120 { 7 } else { 6 },
            strengths: vec![
                "Clear communication".to_string(),
                "Thoughtful response".to_string(),
            ],
            improvements: vec![
                "Could provide more specific examples".to_string(),
            ],
            detailed_feedback: "Good overall response with room for more concrete examples.".to_string(),
            follow_up_suggestions: vec![
                "Practice with more detailed scenarios".to_string(),
            ],
        }
    }
    
    async fn sync_question_to_db(&self, question: &Question) -> Result<(), String> {
        let db = DatabaseManager::new().await
            .map_err(|e| format!("Database connection failed: {}", e))?;
        
        db.insert_interview_question(
            &self.session_id,
            question.number as i32,
            &question.text,
            &question.category,
            &question.difficulty_level,
            question.expected_duration as i32,
        ).await
        .map_err(|e| format!("Failed to store question: {}", e))?;
        
        Ok(())
    }
    
    async fn sync_answer_to_db(&self, answer: &Answer, feedback: &AIFeedback) -> Result<(), String> {
        let db = DatabaseManager::new().await
            .map_err(|e| format!("Database connection failed: {}", e))?;
        
        let question_id = Uuid::parse_str(&answer.question_id)
            .map_err(|_| "Invalid question ID format".to_string())?;
        
        db.insert_interview_answer(
            &question_id,
            &self.session_id,
            Some(&answer.text),
            Some(answer.response_time as i32),
            Some(&feedback.detailed_feedback),
            Some(feedback.score as i32),
        ).await
        .map_err(|e| format!("Failed to store answer: {}", e))?;
        
        Ok(())
    }
    
    fn determine_question_category(&self) -> String {
        match self.current_question_number {
            1..=2 => "introduction".to_string(),
            3..=5 => "behavioral".to_string(),
            6..=8 => "technical".to_string(),
            _ => "situational".to_string(),
        }
    }
    
    fn calculate_expected_duration(&self) -> u32 {
        match self.difficulty.as_str() {
            "beginner" => 3,
            "intermediate" => 5,
            "advanced" => 8,
            _ => 5,
        }
    }
}

// Tauri commands for the interview engine

#[tauri::command]
pub async fn start_interview_session(session_id: String) -> Result<InterviewEngine, String> {
    info!("🎬 Starting interview session: {}", session_id);
    
    let session = crate::session::get_active_session(&session_id).await
        .ok_or_else(|| "Session not active or not found".to_string())?;
    
    let engine = InterviewEngine::new(session_id, session.interview_config).await?;
    
    info!("✅ Interview engine initialized for: {}", engine.job_title);
    
    Ok(engine)
}

#[tauri::command]
pub async fn generate_interview_question(session_id: String) -> Result<Question, String> {
    info!("🎯 Generating new interview question for session: {}", session_id);
    
    let session = crate::session::get_active_session(&session_id).await
        .ok_or_else(|| "Session not active or not found".to_string())?;
    
    let mut engine = InterviewEngine::new(session_id, session.interview_config).await?;
    
    let question = engine.generate_next_question().await?;
    
    info!("✅ Generated question #{}: {}", question.number, question.text.chars().take(50).collect::<String>());
    
    Ok(question)
}

#[tauri::command]
pub async fn submit_interview_answer(
    session_id: String,
    question_id: String,
    answer_text: String,
    response_time: u32,
) -> Result<AIFeedback, String> {
    info!("📝 Submitting answer for session: {} question: {}", session_id, question_id);
    
    let session = crate::session::get_active_session(&session_id).await
        .ok_or_else(|| "Session not active or not found".to_string())?;
    
    let engine = InterviewEngine::new(session_id, session.interview_config).await?;
    
    // Create answer object
    let answer = Answer {
        question_id,
        text: answer_text,
        response_time,
        submitted_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    // For now, create a dummy question - in a real implementation, we'd retrieve it
    let question = Question {
        id: answer.question_id.clone(),
        number: 1,
        text: "Sample question".to_string(),
        category: "general".to_string(),
        difficulty_level: engine.difficulty.clone(),
        expected_duration: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let feedback = engine.evaluate_answer(&question, &answer).await?;
    
    info!("✅ Answer evaluated with score: {}/10", feedback.score);
    
    Ok(feedback)
}
