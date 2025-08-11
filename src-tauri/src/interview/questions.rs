use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Question {
    pub id: String,
    pub number: u32,
    pub text: String,
    pub category: String, // technical, behavioral, situational, introduction
    pub difficulty_level: String,
    pub expected_duration: u32, // in minutes
    pub created_at: u64,
}
