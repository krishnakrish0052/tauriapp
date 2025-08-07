use std::collections::HashMap;
use std::time::Instant;
use log::info;
use regex::Regex;

/// Enhanced transcription result with additional metadata
#[derive(Debug, Clone)]
pub struct EnhancedTranscript {
    pub text: String,
    pub is_final: bool,
    pub confidence: f64,
    pub is_question: bool,
    pub speaker: Option<u8>,
    pub technical_terms: Vec<String>,
    pub start_time: f64,
    pub end_time: f64,
}

/// Represents a single word with its timing and speaker information
#[derive(Debug, Clone, Default)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub speaker: Option<u8>,
    pub confidence: f64,
}

/// Represents a Deepgram transcription alternative, including words and their timings
#[derive(Debug, Clone, Default)]
pub struct DeepgramAlternative {
    pub transcript: String,
    pub confidence: f64,
    pub words: Vec<Word>,
    pub speaker: Option<u8>,
}

/// Intelligent transcription buffer that accumulates and processes text
#[derive(Debug)]
pub struct TranscriptionBuffer {
    // Text accumulation
    interim_results: Vec<DeepgramAlternative>,
    final_results: Vec<DeepgramAlternative>,
    current_utterance_start_time: Option<Instant>,
    
    // Configuration
    confidence_threshold: f64,
    
    // State tracking
    last_update: Instant,
    
    // Question detection
    question_detector: QuestionDetector,
    
    // Technical vocabulary
    technical_vocabulary: HashMap<String, String>,
    
    // Speaker tracking
    current_speaker: Option<u8>,
    speaker_changes: Vec<(Instant, Option<u8>)>,

    // Stats
    word_count: usize,
    final_utterance_count: usize,
    total_duration_ms: u64,
}

impl TranscriptionBuffer {
    pub fn new() -> Self {
        Self {
            interim_results: Vec::new(),
            final_results: Vec::new(),
            current_utterance_start_time: None,
            
            confidence_threshold: 0.6, // Lower threshold for interviews but adaptive
            
            last_update: Instant::now(),
            
            question_detector: QuestionDetector::new(),
            technical_vocabulary: Self::initialize_tech_vocabulary(),
            
            current_speaker: None,
            speaker_changes: Vec::new(),

            word_count: 0,
            final_utterance_count: 0,
            total_duration_ms: 0,
        }
    }

    /// Process new transcription result from Deepgram
    pub fn add_deepgram_result(
        &mut self,
        transcript: String,
        is_final: bool,
        _speech_final: bool,
        confidence: f64,
        words_json: Option<serde_json::Value>,
    ) {
        self.last_update = Instant::now();

        let mut words: Vec<Word> = Vec::new();
        let mut speaker: Option<u8> = None;

        if let Some(words_value) = words_json {
            if let Some(words_arr) = words_value.as_array() {
                for word_val in words_arr {
                    if let Some(word_obj) = word_val.as_object() {
                        let w = word_obj.get("word").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let start = word_obj.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let end = word_obj.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let conf = word_obj.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let s = word_obj.get("speaker").and_then(|v| v.as_u64()).map(|v| v as u8);

                        if speaker.is_none() && s.is_some() {
                            speaker = s;
                        }

                        words.push(Word {
                            word: w,
                            start,
                            end,
                            speaker: s,
                            confidence: conf,
                        });
                    }
                }
            }
        }

        let alternative = DeepgramAlternative {
            transcript,
            confidence,
            words,
            speaker,
        };

        if is_final {
            info!("Final result received: {:?}", alternative.transcript);
            self.final_results.push(alternative);
            self.interim_results.clear(); // Clear interim results once a final is received
            self.final_utterance_count += 1;
            self.word_count += self.final_results.last().map_or(0, |alt| alt.words.len());
            if let Some(last_word) = self.final_results.last().and_then(|alt| alt.words.last()) {
                if let Some(start_time) = self.current_utterance_start_time {
                    self.total_duration_ms += (last_word.end * 1000.0) as u64 - start_time.elapsed().as_millis() as u64;
                }
            }
            self.current_utterance_start_time = None;
        } else {
            // For interim results, we only keep the latest one
            if self.interim_results.is_empty() {
                self.interim_results.push(alternative);
            } else {
                self.interim_results[0] = alternative;
            }
            if self.current_utterance_start_time.is_none() {
                self.current_utterance_start_time = Some(Instant::now());
            }
        }

        // Track speaker changes
        if speaker != self.current_speaker {
            self.current_speaker = speaker;
            self.speaker_changes.push((Instant::now(), speaker));
        }
    }
    
    /// Get the current enhanced transcript, combining final and interim results
    pub fn get_enhanced_transcript(&self) -> EnhancedTranscript {
        let mut full_text = String::new();
        let mut start_time = 0.0;
        let mut end_time = 0.0;
        let mut speaker: Option<u8> = None;
        let mut confidence_sum = 0.0;
        let mut word_count = 0;

        // Combine final results
        for alt in &self.final_results {
            full_text.push_str(&alt.transcript);
            full_text.push(' ');
            if alt.words.first().is_some() && start_time == 0.0 {
                start_time = alt.words.first().unwrap().start;
            }
            if alt.words.last().is_some() {
                end_time = alt.words.last().unwrap().end;
            }
            confidence_sum += alt.confidence;
            word_count += alt.words.len();
            if speaker.is_none() {
                speaker = alt.speaker;
            }
        }

        // Add interim result
        if let Some(interim_alt) = self.interim_results.first() {
            full_text.push_str(&interim_alt.transcript);
            if interim_alt.words.first().is_some() && start_time == 0.0 {
                start_time = interim_alt.words.first().unwrap().start;
            }
            if interim_alt.words.last().is_some() {
                end_time = interim_alt.words.last().unwrap().end;
            }
            confidence_sum += interim_alt.confidence;
            word_count += interim_alt.words.len();
            if speaker.is_none() {
                speaker = interim_alt.speaker;
            }
        }

        let cleaned_text = self.clean_text(&full_text);
        let is_question = self.question_detector.is_question(&cleaned_text);
        let technical_terms = self.extract_technical_terms(&cleaned_text);
        let avg_confidence = if word_count > 0 { confidence_sum / word_count as f64 } else { 0.0 };

        EnhancedTranscript {
            text: cleaned_text,
            is_final: self.interim_results.is_empty(), // If no interim, it's final
            confidence: avg_confidence,
            is_question,
            speaker,
            technical_terms,
            start_time,
            end_time,
        }
    }

    /// Get statistics about the transcription
    pub fn get_stats(&self) -> TranscriptionStats {
        TranscriptionStats {
            word_count: self.word_count,
            final_utterance_count: self.final_utterance_count,
            total_duration_ms: self.total_duration_ms,
            speaker_changes: self.speaker_changes.len(),
            questions_detected: 0, // This needs to be calculated from final_results if needed
        }
    }

    /// Clear all accumulated text and reset state
    pub fn clear(&mut self) {
        self.interim_results.clear();
        self.final_results.clear();
        self.current_utterance_start_time = None;
        self.last_update = Instant::now();
        self.current_speaker = None;
        self.speaker_changes.clear();
        self.word_count = 0;
        self.final_utterance_count = 0;
        self.total_duration_ms = 0;
    }

    // Private helper methods

    fn clean_text(&self, text: &str) -> String {
        // Basic text cleaning
        text.trim()
            .chars()
            .filter(|c| c.is_ascii() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn extract_technical_terms(&self, text: &str) -> Vec<String> {
        let mut terms = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in words {
            let clean_word = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            
            if self.technical_vocabulary.contains_key(&clean_word) {
                terms.push(clean_word);
            }
        }
        
        terms
    }

    fn initialize_tech_vocabulary() -> HashMap<String, String> {
        let mut vocab = HashMap::new();
        
        // Programming terms
        for term in &[
            "javascript", "python", "rust", "java", "react", "node", "api", "database",
            "algorithm", "framework", "library", "repository", "github", "git",
            "frontend", "backend", "fullstack", "microservices", "kubernetes",
            "docker", "aws", "azure", "cloud", "devops", "ci/cd", "testing"
        ] {
            vocab.insert(term.to_string(), "programming".to_string());
        }
        
        // General tech terms
        for term in &[
            "artificial intelligence", "machine learning", "data science",
            "blockchain", "cryptocurrency", "cybersecurity", "automation",
            "scalability", "performance", "optimization", "analytics"
        ] {
            vocab.insert(term.to_string(), "technology".to_string());
        }
        
        vocab
    }
}

/// Statistics about the transcription session
#[derive(Debug, Clone)]
pub struct TranscriptionStats {
    pub word_count: usize,
    pub final_utterance_count: usize,
    pub total_duration_ms: u64,
    pub speaker_changes: usize,
    pub questions_detected: usize,
}

#[derive(Debug)]
struct QuestionDetector {
    question_words: Vec<String>,
    question_patterns: Vec<Regex>,
}

impl QuestionDetector {
    fn new() -> Self {
        Self {
            question_words: vec![
                "what".to_string(), "how".to_string(), "why".to_string(),
                "when".to_string(), "where".to_string(), "who".to_string(),
                "which".to_string(), "can".to_string(), "could".to_string(),
                "would".to_string(), "should".to_string(), "do".to_string(),
                "does".to_string(), "did".to_string(), "will".to_string(),
                "are".to_string(), "is".to_string(), "was".to_string(),
                "were".to_string(), "have".to_string(), "has".to_string(),
                "had".to_string(),
            ],
            question_patterns: vec![
                Regex::new(r"\?+$").unwrap(), // Ends with question mark
                Regex::new(r"^(can|could|would|should|do|does|did|will|are|is|was|were|have|has|had)\s").unwrap(), // Starts with auxiliary verbs
            ],
        }
    }
    
    fn is_question(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        
        // Check for question mark
        if text.ends_with('?') {
            return true;
        }
        
        // Check question patterns
        for pattern in &self.question_patterns {
            if pattern.is_match(&text_lower) {
                return true;
            }
        }
        
        // Check if starts with question words
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        if let Some(first_word) = words.first() {
            if self.question_words.contains(&first_word.to_string()) {
                return true;
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transcription_buffer() {
        let mut buffer = TranscriptionBuffer::new();
        
        // Simulate Deepgram interim results
        buffer.add_deepgram_result(
            "Hello world".to_string(),
            false,
            false,
            0.9,
            Some(serde_json::json!([
                {"word": "Hello", "start": 0.0, "end": 0.5, "speaker": 0, "confidence": 0.95},
                {"word": "world", "start": 0.6, "end": 1.0, "speaker": 0, "confidence": 0.85}
            ])),
        );
        let enhanced = buffer.get_enhanced_transcript();
        assert_eq!(enhanced.text, "Hello world");
        assert!(!enhanced.is_final);
        assert_eq!(enhanced.speaker, Some(0));

        // Simulate Deepgram final result
        buffer.add_deepgram_result(
            "This is a test.".to_string(),
            true,
            true,
            0.98,
            Some(serde_json::json!([
                {"word": "This", "start": 1.1, "end": 1.3, "speaker": 0, "confidence": 0.99},
                {"word": "is", "start": 1.3, "end": 1.4, "speaker": 0, "confidence": 0.98},
                {"word": "a", "start": 1.4, "end": 1.5, "speaker": 0, "confidence": 0.97},
                {"word": "test.", "start": 1.5, "end": 2.0, "speaker": 0, "confidence": 0.96}
            ])),
        );
        let enhanced = buffer.get_enhanced_transcript();
        assert_eq!(enhanced.text, "Hello world This is a test.");
        assert!(enhanced.is_final);
        assert_eq!(enhanced.speaker, Some(0));

        let stats = buffer.get_stats();
        assert_eq!(stats.final_utterance_count, 1);
        assert_eq!(stats.word_count, 6);
    }

    #[test]
    fn test_speaker_change() {
        let mut buffer = TranscriptionBuffer::new();

        buffer.add_deepgram_result("Hello".to_string(), true, true, 0.9, Some(serde_json::json!([{"word": "Hello", "start": 0.0, "end": 0.5, "speaker": 0}])));
        buffer.add_deepgram_result("Hi there".to_string(), true, true, 0.9, Some(serde_json::json!([{"word": "Hi", "start": 0.6, "end": 0.8, "speaker": 1}])));

        let stats = buffer.get_stats();
        assert_eq!(stats.speaker_changes, 2); // Initial speaker + one change
    }

    #[test]
    fn test_question_detection() {
        let detector = QuestionDetector::new();
        assert!(detector.is_question("What is your name?"));
        assert!(detector.is_question("How are you doing"));
        assert!(detector.is_question("Can you help me"));
        assert!(detector.is_question("Do you understand"));
        assert!(!detector.is_question("I understand the question."));
        assert!(!detector.is_question("This is a statement."));
    }
}
