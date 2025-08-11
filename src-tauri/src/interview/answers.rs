use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    pub question_id: String,
    pub text: String,
    pub response_time: u32, // in seconds
    pub submitted_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AIFeedback {
    pub score: u8, // 1-10
    pub strengths: Vec<String>,
    pub improvements: Vec<String>,
    pub detailed_feedback: String,
    pub follow_up_suggestions: Vec<String>,
}
