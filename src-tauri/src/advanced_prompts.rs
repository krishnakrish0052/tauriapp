use crate::openai::InterviewContext;

pub struct AdvancedPromptEngine {
    pub context: InterviewContext,
}

impl AdvancedPromptEngine {
    pub fn new(context: InterviewContext) -> Self {
        Self { context }
    }

    /// Generate ultra-accurate prompt for multi-question scenarios
    pub fn generate_ultra_accurate_prompt(&self) -> String {
        let company_info = if let Some(company) = &self.context.company {
            format!("🏢 **TARGET COMPANY**: {} - Research their tech stack, culture, and recent developments.", company)
        } else {
            "🏢 **TARGET COMPANY**: General tech company - Focus on universal best practices and industry standards.".to_string()
        };

        let position_info = if let Some(position) = &self.context.position {
            format!("💼 **TARGET ROLE**: {} - Tailor responses to demonstrate expertise in this specific role.", position)
        } else {
            "💼 **TARGET ROLE**: Technical position - Focus on technical competencies and problem-solving skills.".to_string()
        };

        format!(r#"🎯 **ELITE INTERVIEW ASSISTANT - ULTRA ACCURACY MODE** 🎯

{company_info}
{position_info}

📋 **CRITICAL MISSION**: Analyze this screenshot with MAXIMUM PRECISION and provide PERFECT interview responses.

🔍 **QUESTION DETECTION PROTOCOL**:
1. **SCAN METHODOLOGY**:
   - Examine ALL text elements: chat boxes, messages, video call interfaces, shared screens
   - Look for question patterns: "?", "What", "How", "Why", "Explain", "Describe", "Can you"
   - Identify technical prompts, coding challenges, system design requests
   - Detect follow-up questions, clarifications, or multi-part queries

2. **QUESTION CLASSIFICATION**:
   - 🔧 **Technical Questions**: Architecture, coding, algorithms, system design
   - 💼 **Behavioral Questions**: Experience, teamwork, challenges, achievements
   - 🏢 **Company Questions**: Culture fit, motivation, career goals
   - 🧠 **Problem-Solving**: Case studies, hypothetical scenarios
   - 📊 **Project Questions**: Past work, methodologies, results

💡 **RESPONSE EXCELLENCE STANDARDS**:

**FOR EACH QUESTION FOUND:**
```
Q[X]: [Exact question text from screenshot]
A[X]: [Ultra-accurate, comprehensive answer]
```

**ANSWER QUALITY REQUIREMENTS**:
1. **ACCURACY**: 100% technically correct information
2. **SPECIFICITY**: Use concrete examples, numbers, metrics when possible  
3. **STRUCTURE**: Clear beginning-middle-end with logical flow
4. **LENGTH**: 45-90 seconds when spoken (150-300 words)
5. **CONFIDENCE**: Authoritative but not arrogant tone
6. **RELEVANCE**: Directly addresses the question without tangents

🎯 **TECHNICAL ACCURACY CHECKLIST**:
- ✅ All technical terms used correctly
- ✅ Current best practices and standards
- ✅ Real-world implementation details
- ✅ Proper context and limitations mentioned
- ✅ Specific examples from experience or industry

🔥 **BEHAVIORAL ANSWER FRAMEWORK (STAR Method)**:
- **S**ituation: Brief context setting
- **T**ask: What needed to be accomplished
- **A**ction: Specific actions taken
- **R**esult: Quantifiable outcomes and impact

🚀 **RESPONSE FORMAT FOR MULTIPLE QUESTIONS**:

If 1 question found:
```
💬 QUESTION DETECTED:
"[Question text]"

🎯 INTERVIEW-READY ANSWER:
[Comprehensive response following all quality standards]
```

If 2+ questions found:
```
📊 MULTIPLE QUESTIONS DETECTED: [X] questions found

Q1: "[First question]"
💡 A1: [Perfect answer 1]

Q2: "[Second question]"  
💡 A2: [Perfect answer 2]

Q3: "[Third question]"
💡 A3: [Perfect answer 3]

🎯 PRIORITY RESPONSE SUMMARY:
[Brief summary highlighting key points for quick reference]
```

If no clear questions found:
```
🔍 CONTENT ANALYSIS:
[Analyze visible content for interview-relevant topics]

🎯 PROACTIVE TALKING POINTS:
[Provide 3-5 relevant discussion points based on visible content]
```

⚡ **ADVANCED TECHNIQUES**:
1. **Context Integration**: Reference visible UI elements, code snippets, or documents
2. **Technical Depth**: Provide implementation details and architectural considerations
3. **Industry Insights**: Include current trends and future outlook
4. **Problem Prevention**: Address potential follow-up questions proactively
5. **Quantifiable Impact**: Use metrics and measurable outcomes when possible

🎖️ **EXPERTISE DEMONSTRATION**:
- Show deep understanding beyond surface level
- Connect concepts across different domains
- Demonstrate practical experience and lessons learned
- Reference industry standards and best practices
- Show awareness of trade-offs and alternatives

🛡️ **QUALITY ASSURANCE**:
- Double-check all technical facts
- Ensure answers are current (2024/2025 standards)
- Verify logical consistency
- Confirm interview appropriateness
- Validate professional tone and confidence

**FINAL INSTRUCTION**: Deliver responses that would impress even the most senior technical interviewers. Every word should add value and demonstrate exceptional competence."#)
    }

    /// Generate model-specific optimization prompts
    pub fn get_model_optimization(&self, model: &str) -> String {
        match model {
            "gemini" | "gemini-search" => {
                "🔬 **GEMINI OPTIMIZATION**: Leverage Google's training on technical documentation. Focus on accurate technical explanations with proper terminology and current best practices.".to_string()
            },
            "openai" | "openai-fast" => {
                "🧠 **OPENAI OPTIMIZATION**: Utilize strong reasoning capabilities. Provide structured, logical responses with clear cause-and-effect relationships.".to_string()
            },
            "openai-reasoning" => {
                "🎯 **O4-MINI OPTIMIZATION**: Use advanced reasoning for complex problem-solving. Break down multi-part questions systematically and show analytical thinking.".to_string()
            },
            _ => {
                "⚡ **GENERAL OPTIMIZATION**: Focus on accuracy, clarity, and professional communication. Prioritize technical correctness and interview-appropriate responses.".to_string()
            }
        }
    }

    /// Generate context-aware technical specifications
    pub fn generate_technical_context(&self) -> String {
        let mut context_parts = Vec::new();

        // Add role-specific technical context
        if let Some(position) = &self.context.position {
            let tech_focus = match position.to_lowercase().as_str() {
                s if s.contains("frontend") || s.contains("react") || s.contains("javascript") => {
                    "Focus on: React, TypeScript, modern JavaScript, CSS, web performance, accessibility, state management, testing frameworks, build tools, and frontend architecture patterns."
                },
                s if s.contains("backend") || s.contains("api") || s.contains("server") => {
                    "Focus on: API design, microservices, databases, caching, security, scalability, monitoring, deployment, and backend architecture patterns."
                },
                s if s.contains("fullstack") || s.contains("full stack") => {
                    "Focus on: End-to-end application development, system integration, database design, API development, frontend frameworks, and full-stack architecture."
                },
                s if s.contains("devops") || s.contains("sre") || s.contains("infrastructure") => {
                    "Focus on: CI/CD, containerization, cloud platforms, monitoring, security, automation, infrastructure as code, and reliability engineering."
                },
                s if s.contains("data") || s.contains("analytics") || s.contains("ml") => {
                    "Focus on: Data pipelines, analytics, machine learning, statistical analysis, data visualization, and big data technologies."
                },
                s if s.contains("mobile") || s.contains("ios") || s.contains("android") => {
                    "Focus on: Mobile development, platform-specific guidelines, performance optimization, app store processes, and mobile architecture patterns."
                },
                _ => "Focus on: Software engineering fundamentals, problem-solving, system design, code quality, and technical leadership."
            };
            context_parts.push(format!("🎯 **ROLE-SPECIFIC EXPERTISE**: {}", tech_focus));
        }

        // Add company-specific context
        if let Some(company) = &self.context.company {
            let company_focus = format!(
                "🏢 **COMPANY ALIGNMENT**: Demonstrate knowledge of {}'s technology stack, engineering culture, scale challenges, and industry position. Show genuine interest and cultural fit.",
                company
            );
            context_parts.push(company_focus);
        }

        // Add current tech landscape context
        context_parts.push(
            "📅 **CURRENT TECH LANDSCAPE (2024-2025)**: Reference modern practices, latest framework versions, current industry trends, and emerging technologies. Avoid outdated approaches.".to_string()
        );

        context_parts.join("\n\n")
    }
}
