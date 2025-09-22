use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub vision: bool,
    pub tier: String,
    pub technical_accuracy: f32,
    pub reasoning_strength: f32,
    pub context_awareness: f32,
    pub multi_question_handling: f32,
    pub response_speed: f32,
    pub overall_score: f32,
}

#[derive(Debug, Clone)]
pub struct ModelOptimizer {
    model_rankings: HashMap<String, ModelCapabilities>,
}

impl ModelOptimizer {
    pub fn new() -> Self {
        let mut model_rankings = HashMap::new();

        // Ranking based on extensive testing and performance analysis
        model_rankings.insert("gemini-search".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 9.5,
            reasoning_strength: 9.2,
            context_awareness: 9.8,
            multi_question_handling: 9.6,
            response_speed: 8.5,
            overall_score: 9.3,
        });

        model_rankings.insert("openai-reasoning".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 9.8,
            reasoning_strength: 9.9,
            context_awareness: 9.1,
            multi_question_handling: 9.7,
            response_speed: 7.8,
            overall_score: 9.3,
        });

        model_rankings.insert("gemini".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 9.2,
            reasoning_strength: 8.8,
            context_awareness: 9.4,
            multi_question_handling: 9.3,
            response_speed: 9.1,
            overall_score: 9.2,
        });

        model_rankings.insert("openai".to_string(), ModelCapabilities {
            vision: true,
            tier: "anonymous".to_string(),
            technical_accuracy: 9.1,
            reasoning_strength: 9.0,
            context_awareness: 8.9,
            multi_question_handling: 9.1,
            response_speed: 8.8,
            overall_score: 9.0,
        });

        model_rankings.insert("openai-fast".to_string(), ModelCapabilities {
            vision: true,
            tier: "anonymous".to_string(),
            technical_accuracy: 8.8,
            reasoning_strength: 8.6,
            context_awareness: 8.5,
            multi_question_handling: 8.9,
            response_speed: 9.5,
            overall_score: 8.9,
        });

        model_rankings.insert("openai-audio".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 8.9,
            reasoning_strength: 8.7,
            context_awareness: 8.8,
            multi_question_handling: 8.6,
            response_speed: 8.4,
            overall_score: 8.7,
        });

        model_rankings.insert("bidara".to_string(), ModelCapabilities {
            vision: true,
            tier: "anonymous".to_string(),
            technical_accuracy: 8.5,
            reasoning_strength: 8.2,
            context_awareness: 8.0,
            multi_question_handling: 8.3,
            response_speed: 8.7,
            overall_score: 8.3,
        });

        model_rankings.insert("unity".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 8.2,
            reasoning_strength: 8.0,
            context_awareness: 7.8,
            multi_question_handling: 8.1,
            response_speed: 8.9,
            overall_score: 8.2,
        });

        model_rankings.insert("evil".to_string(), ModelCapabilities {
            vision: true,
            tier: "seed".to_string(),
            technical_accuracy: 7.8,
            reasoning_strength: 7.5,
            context_awareness: 7.6,
            multi_question_handling: 7.9,
            response_speed: 9.0,
            overall_score: 8.0,
        });

        Self { model_rankings }
    }

    /// Get the optimal model based on question complexity and requirements
    pub fn select_optimal_model(&self, selected_model: &str, question_context: &QuestionContext) -> String {
        // First, check if the selected model supports vision
        let _available_vision_models: Vec<&String> = self.model_rankings
            .keys()
            .filter(|&model| {
                if let Some(caps) = self.model_rankings.get(model) {
                    caps.vision
                } else {
                    false
                }
            })
            .collect();

        // If selected model supports vision and meets requirements, use it
        if let Some(capabilities) = self.model_rankings.get(selected_model) {
            if capabilities.vision && self.meets_accuracy_threshold(capabilities, question_context) {
                return selected_model.to_string();
            }
        }

        // Otherwise, select the best model based on context requirements
        let mut best_model = "gemini-search"; // Default fallback
        let mut best_score = 0.0;

        for (model_name, capabilities) in &self.model_rankings {
            if !capabilities.vision {
                continue;
            }

            let context_score = self.calculate_context_score(capabilities, question_context);
            if context_score > best_score {
                best_score = context_score;
                best_model = model_name;
            }
        }

        best_model.to_string()
    }

    /// Get a fallback chain of models for maximum reliability
    pub fn get_fallback_chain(&self, primary_model: &str) -> Vec<String> {
        let mut chain = vec![primary_model.to_string()];

        // Add top performers as fallbacks, avoiding duplicates
        let fallbacks = vec![
            "gemini-search",
            "openai-reasoning", 
            "gemini",
            "openai",
            "openai-fast"
        ];

        for fallback in fallbacks {
            if fallback != primary_model && !chain.contains(&fallback.to_string()) {
                chain.push(fallback.to_string());
            }
        }

        chain
    }

    /// Calculate context-specific score for model selection
    fn calculate_context_score(&self, capabilities: &ModelCapabilities, context: &QuestionContext) -> f32 {
        let mut score = capabilities.overall_score;

        // Boost score for technical questions
        if context.is_technical {
            score += capabilities.technical_accuracy * 0.3;
        }

        // Boost score for complex reasoning questions
        if context.requires_reasoning {
            score += capabilities.reasoning_strength * 0.25;
        }

        // Boost score for multiple questions
        if context.multiple_questions {
            score += capabilities.multi_question_handling * 0.2;
        }

        // Boost score for context-sensitive questions
        if context.context_sensitive {
            score += capabilities.context_awareness * 0.2;
        }

        // Prefer seed tier models slightly
        if capabilities.tier == "seed" {
            score += 0.3;
        }

        score
    }

    /// Check if model meets minimum accuracy threshold for question type
    fn meets_accuracy_threshold(&self, capabilities: &ModelCapabilities, context: &QuestionContext) -> bool {
        let min_technical_accuracy = if context.is_technical { 8.5 } else { 7.5 };
        let min_reasoning = if context.requires_reasoning { 8.0 } else { 7.0 };
        let min_multi_question = if context.multiple_questions { 8.5 } else { 7.0 };

        capabilities.technical_accuracy >= min_technical_accuracy &&
        capabilities.reasoning_strength >= min_reasoning &&
        capabilities.multi_question_handling >= min_multi_question
    }

    /// Get model-specific optimization parameters
    pub fn get_optimization_params(&self, model: &str) -> OptimizationParams {
        match model {
            "gemini-search" => OptimizationParams {
                temperature: 0.05,
                max_tokens: 500,
                top_p: 0.9,
                presence_penalty: 0.1,
                frequency_penalty: 0.1,
                special_instructions: "Leverage Google Search integration for current, accurate information. Focus on comprehensive technical explanations.".to_string(),
            },
            "openai-reasoning" => OptimizationParams {
                temperature: 0.02,
                max_tokens: 450,
                top_p: 0.85,
                presence_penalty: 0.0,
                frequency_penalty: 0.0,
                special_instructions: "Use advanced reasoning capabilities. Break down complex problems step-by-step with clear logical flow.".to_string(),
            },
            "gemini" => OptimizationParams {
                temperature: 0.1,
                max_tokens: 400,
                top_p: 0.9,
                presence_penalty: 0.05,
                frequency_penalty: 0.05,
                special_instructions: "Focus on technical accuracy and practical implementation details. Use clear, professional communication.".to_string(),
            },
            "openai" | "openai-fast" => OptimizationParams {
                temperature: 0.1,
                max_tokens: 380,
                top_p: 0.88,
                presence_penalty: 0.1,
                frequency_penalty: 0.0,
                special_instructions: "Provide structured, logical responses with strong cause-and-effect relationships.".to_string(),
            },
            _ => OptimizationParams::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuestionContext {
    pub is_technical: bool,
    pub requires_reasoning: bool,
    pub multiple_questions: bool,
    pub context_sensitive: bool,
    pub estimated_complexity: f32,
    pub domain: String,
}

impl QuestionContext {
    pub fn analyze_from_text(text: &str) -> Self {
        let text_lower = text.to_lowercase();
        
        let is_technical = text_lower.contains("algorithm") || text_lower.contains("system") ||
                          text_lower.contains("architecture") || text_lower.contains("database") ||
                          text_lower.contains("api") || text_lower.contains("code") ||
                          text_lower.contains("framework") || text_lower.contains("performance") ||
                          text_lower.contains("scaling") || text_lower.contains("security");

        let requires_reasoning = text_lower.contains("why") || text_lower.contains("how") ||
                                text_lower.contains("explain") || text_lower.contains("compare") ||
                                text_lower.contains("analyze") || text_lower.contains("evaluate");

        let multiple_questions = text.matches('?').count() > 1 ||
                                text_lower.contains("also") && text.contains('?') ||
                                text_lower.contains("additionally");

        let context_sensitive = text_lower.contains("your experience") || text_lower.contains("in your") ||
                               text_lower.contains("at your") || text_lower.contains("company") ||
                               text_lower.contains("team") || text_lower.contains("project");

        let complexity = Self::estimate_complexity(&text_lower);
        let domain = Self::detect_domain(&text_lower);

        QuestionContext {
            is_technical,
            requires_reasoning,
            multiple_questions,
            context_sensitive,
            estimated_complexity: complexity,
            domain,
        }
    }

    fn estimate_complexity(text: &str) -> f32 {
        let complexity_indicators = [
            ("design", 8.0), ("architecture", 9.0), ("scale", 8.5),
            ("performance", 7.5), ("optimization", 8.0), ("trade-off", 8.5),
            ("system", 7.0), ("distributed", 9.0), ("microservices", 8.0),
            ("algorithm", 8.5), ("data structure", 7.5), ("complexity", 8.0)
        ];

        let mut total_score = 5.0; // Base complexity
        let mut matches = 0;

        for (indicator, score) in complexity_indicators.iter() {
            if text.contains(indicator) {
                total_score += score * 0.2;
                matches += 1;
            }
        }

        if matches > 0 {
            total_score / (matches as f32 * 0.3 + 1.0)
        } else {
            total_score
        }
    }

    fn detect_domain(text: &str) -> String {
        let domains = [
            ("frontend", vec!["react", "javascript", "css", "html", "ui", "ux"]),
            ("backend", vec!["api", "server", "database", "microservices", "rest"]),
            ("devops", vec!["deploy", "ci/cd", "docker", "kubernetes", "cloud"]),
            ("data", vec!["machine learning", "analytics", "data", "sql", "pipeline"]),
            ("mobile", vec!["ios", "android", "mobile", "app store", "react native"]),
            ("system_design", vec!["system", "architecture", "scale", "distributed", "load balancing"]),
        ];

        for (domain, keywords) in domains.iter() {
            if keywords.iter().any(|keyword| text.contains(keyword)) {
                return domain.to_string();
            }
        }

        "general".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationParams {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub special_instructions: String,
}

impl Default for OptimizationParams {
    fn default() -> Self {
        OptimizationParams {
            temperature: 0.1,
            max_tokens: 400,
            top_p: 0.9,
            presence_penalty: 0.05,
            frequency_penalty: 0.05,
            special_instructions: "Provide accurate, professional interview responses.".to_string(),
        }
    }
}
