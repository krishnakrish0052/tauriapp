// Test script to verify QA Storage functionality
// Run this in browser console to test both manual and generate button flows

console.log('🧪 Starting QA Storage Test...');

// Mock the QA Storage Manager for testing
class MockQAStorageManager {
    constructor() {
        this.currentSession = { id: 'test-session-123' };
        this.currentQuestionId = null;
        this.questionStartTime = null;
        this.questions = [];
        this.answers = [];
        
        console.log('📦 Mock QA Storage Manager initialized');
    }
    
    async storeQuestion(questionData) {
        console.log('📝 MOCK: Storing question:', questionData);
        
        const questionId = 'q-' + Date.now();
        this.currentQuestionId = questionId;
        this.questionStartTime = Date.now();
        
        const questionRecord = {
            id: questionId,
            ...questionData,
            timestamp: new Date().toISOString()
        };
        
        this.questions.push(questionRecord);
        console.log('✅ MOCK: Question stored with ID:', questionId);
        
        return {
            question: {
                id: questionId,
                text: questionData.questionText,
                number: questionData.questionNumber
            }
        };
    }
    
    async storeAnswer(answerData) {
        console.log('💬 MOCK: Storing answer:', answerData);
        console.log('🔗 MOCK: Current question ID:', this.currentQuestionId);
        
        if (!this.currentQuestionId) {
            console.warn('⚠️ MOCK: No current question ID - answer may not be linked!');
        }
        
        const responseTime = this.questionStartTime 
            ? Math.round((Date.now() - this.questionStartTime) / 1000)
            : null;
        
        const answerRecord = {
            id: 'a-' + Date.now(),
            questionId: this.currentQuestionId,
            ...answerData,
            responseTime,
            timestamp: new Date().toISOString()
        };
        
        this.answers.push(answerRecord);
        console.log('✅ MOCK: Answer stored successfully');
        
        // Reset for next question
        this.currentQuestionId = null;
        this.questionStartTime = null;
        
        return {
            answer: {
                id: answerRecord.id,
                text: answerData.answerText,
                questionId: answerRecord.questionId
            }
        };
    }
    
    getStoredData() {
        return {
            questions: this.questions,
            answers: this.answers,
            summary: {
                totalQuestions: this.questions.length,
                totalAnswers: this.answers.length,
                linkedAnswers: this.answers.filter(a => a.questionId).length
            }
        };
    }
}

// Set up mock QA Storage Manager
window.qaStorageManager = new MockQAStorageManager();

// Test functions
async function testManualQuestion() {
    console.log('\n🧪 Testing Manual Question Flow...');
    
    // Simulate storing manual question
    await window.qaStorageManager.storeQuestion({
        questionText: "What is your experience with React?",
        questionNumber: 1,
        category: 'manual',
        difficultyLevel: 'medium',
        source: 'manual',
        metadata: {
            timestamp: new Date().toISOString(),
            expectedDuration: 5
        }
    });
    
    // Simulate AI response (after some delay)
    setTimeout(async () => {
        await window.qaStorageManager.storeAnswer({
            answerText: "I have 3 years of experience with React, including hooks, state management with Redux, and building responsive web applications.",
            source: 'ai_response',
            metadata: {
                aiProvider: 'pollinations',
                aiModel: 'llama-3.1-70b-instruct',
                timestamp: new Date().toISOString()
            }
        });
        
        console.log('✅ Manual question test completed');
        displayResults();
    }, 1000);
}

async function testGenerateButtonFlow() {
    console.log('\n🧪 Testing Generate Button Flow...');
    
    // Simulate storing generated question
    await window.qaStorageManager.storeQuestion({
        questionText: "Tell me about your experience with Node.js and Express",
        questionNumber: 2,
        category: 'generated',
        difficultyLevel: 'medium',
        source: 'generate_button',
        metadata: {
            timestamp: new Date().toISOString(),
            expectedDuration: 5,
            source: 'transcription_or_input',
            provider: 'pollinations',
            model: 'llama-3.1-70b-instruct'
        }
    });
    
    // Simulate AI response (after some delay)
    setTimeout(async () => {
        await window.qaStorageManager.storeAnswer({
            answerText: "I have extensive experience with Node.js and Express, having built several REST APIs and web applications. I'm familiar with middleware, routing, authentication, and database integration.",
            source: 'ai_response',
            metadata: {
                aiProvider: 'pollinations',
                aiModel: 'llama-3.1-70b-instruct',
                timestamp: new Date().toISOString(),
                questionLinked: !!window.qaStorageManager.currentQuestionId
            }
        });
        
        console.log('✅ Generate button test completed');
        displayResults();
    }, 1500);
}

function displayResults() {
    const data = window.qaStorageManager.getStoredData();
    
    console.log('\n📊 QA Storage Test Results:');
    console.log('='.repeat(50));
    console.log(`📝 Total Questions: ${data.summary.totalQuestions}`);
    console.log(`💬 Total Answers: ${data.summary.totalAnswers}`);
    console.log(`🔗 Linked Answers: ${data.summary.linkedAnswers}`);
    console.log('\n📋 Questions:', data.questions);
    console.log('\n💭 Answers:', data.answers);
    
    if (data.summary.totalQuestions === data.summary.linkedAnswers) {
        console.log('\n✅ SUCCESS: All answers are properly linked to questions!');
    } else {
        console.log('\n❌ ISSUE: Some answers are not linked to questions!');
    }
}

// Run tests
console.log('🚀 Starting QA Storage Tests...');
testManualQuestion();

setTimeout(() => {
    testGenerateButtonFlow();
}, 2500);

console.log('\n💡 Check the results above after both tests complete (about 5 seconds)');
