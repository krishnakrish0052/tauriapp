// Real-time Question and Answer Storage Manager
// Handles direct real-time storage of interview Q&A to the backend

class QAStorageManager {
    constructor() {
        this.currentSession = null;
        this.authToken = null;
        this.userId = null;
        this.apiBaseUrl = 'http://localhost:3001/api'; // Backend API URL
        this.currentQuestionId = null;
        this.questionStartTime = null;
        
        console.log('🗃️ QA Storage Manager initialized');
    }

    // Initialize with session data
    initialize(sessionData, token, userId) {
        this.currentSession = sessionData;
        this.authToken = token;
        this.userId = userId;
        
        console.log('🔗 QA Storage Manager connected to session:', sessionData.id);
    }


    // Store question immediately (manual or transcribed)
    async storeQuestion(questionData) {
        const {
            questionText,
            questionNumber = 1,
            category = 'general',
            difficultyLevel = 'medium',
            source = 'manual', // 'manual' or 'transcribed'
            metadata = {}
        } = questionData;

        if (!questionText || !questionText.trim()) {
            console.warn('⚠️ Empty question text, skipping storage');
            return null;
        }

        if (!this.currentSession) {
            throw new Error('No active session - cannot store question');
        }

        this.questionStartTime = Date.now();
        
        const questionPayload = {
            questionText: questionText.trim(),
            questionNumber,
            category,
            difficultyLevel,
            metadata: {
                source,
                timestamp: new Date().toISOString(),
                ...metadata
            }
        };

        console.log(`📝 Storing question ${questionNumber}: "${questionText.substring(0, 50)}..."`, {
            source,
            category,
            difficulty: difficultyLevel
        });

        const result = await this.sendQuestionToServer(questionPayload);
        this.currentQuestionId = result.question.id;
        console.log('✅ Question stored successfully:', result.question.id);
        return result;
    }

    // Store answer immediately after user provides response
    async storeAnswer(answerData) {
        const {
            answerText,
            questionId = this.currentQuestionId,
            source = 'manual', // 'manual' or 'transcribed'
            metadata = {}
        } = answerData;

        if (!answerText || !answerText.trim()) {
            console.warn('⚠️ Empty answer text, skipping storage');
            return null;
        }

        if (!this.currentSession) {
            throw new Error('No active session - cannot store answer');
        }

        // Calculate response time if we have question start time
        const responseTime = this.questionStartTime 
            ? Math.round((Date.now() - this.questionStartTime) / 1000)
            : null;

        const answerPayload = {
            answerText: answerText.trim(),
            questionId,
            responseTime,
            metadata: {
                source,
                timestamp: new Date().toISOString(),
                responseTimeMs: Date.now() - (this.questionStartTime || Date.now()),
                ...metadata
            }
        };

        console.log(`💬 Storing answer: "${answerText.substring(0, 50)}..."`, {
            responseTime: responseTime ? `${responseTime}s` : 'unknown',
            source,
            questionId
        });

        const result = await this.sendAnswerToServer(answerPayload);
        console.log('✅ Answer stored successfully:', result.answer.id);
        
        // Reset question tracking
        this.currentQuestionId = null;
        this.questionStartTime = null;
        
        return result;
    }

    // Send question to server using Tauri command
    async sendQuestionToServer(questionPayload) {
        // Use Tauri command instead of HTTP API
        const questionId = await window.safeInvoke('save_interview_question', {
            sessionId: this.currentSession.id,
            questionNumber: questionPayload.questionNumber,
            questionText: questionPayload.questionText,
            category: questionPayload.category,
            difficultyLevel: questionPayload.difficultyLevel,
            expectedDuration: questionPayload.metadata?.expectedDuration || 5
        });
        
        return {
            question: {
                id: questionId,
                text: questionPayload.questionText,
                number: questionPayload.questionNumber
            }
        };
    }

    // Send answer to server using Tauri command
    async sendAnswerToServer(answerPayload) {
        if (!answerPayload.questionId) {
            throw new Error('Question ID is required to store answer');
        }
        
        // Use Tauri command instead of HTTP API
        const answerId = await window.safeInvoke('save_interview_answer', {
            sessionId: this.currentSession.id,
            questionId: answerPayload.questionId,
            answerText: answerPayload.answerText,
            responseTime: answerPayload.responseTime || 0,
            aiFeedback: null, // Will be added later if AI evaluation is done
            aiScore: null     // Will be added later if AI evaluation is done
        });
        
        return {
            answer: {
                id: answerId,
                text: answerPayload.answerText,
                questionId: answerPayload.questionId,
                responseTime: answerPayload.responseTime
            }
        };
    }

    // Cleanup when session ends
    cleanup() {
        console.log('🧹 Cleaning up QA Storage Manager');
        
        // Reset session data
        this.currentSession = null;
        this.currentQuestionId = null;
        this.questionStartTime = null;
    }
}

// Global instance
window.qaStorageManager = new QAStorageManager();

console.log('✅ QA Storage Manager loaded successfully');
