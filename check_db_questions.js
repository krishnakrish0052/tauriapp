const { Client } = require('pg');

async function checkQuestions() {
    const client = new Client({
        host: 'localhost',
        port: 5432,
        database: 'mockmate_db',
        user: 'mockmate_user',
        password: 'mockmate_2024!', // From .env file
    });

    try {
        await client.connect();
        console.log('✅ Connected to database');

        // Check recent questions
        const result = await client.query(`
            SELECT 
                id, 
                session_id, 
                message_type, 
                content, 
                metadata->>'questionNumber' as question_num,
                metadata->>'category' as category,
                metadata->>'difficulty' as difficulty,
                metadata->>'source' as source,
                timestamp 
            FROM interview_messages 
            WHERE message_type = 'question' 
            ORDER BY timestamp DESC 
            LIMIT 5
        `);

        console.log('\n📋 Recent Questions in Database:');
        console.log('===============================');
        
        if (result.rows.length === 0) {
            console.log('❌ No questions found in database');
        } else {
            result.rows.forEach((row, index) => {
                console.log(`\n${index + 1}. Question ID: ${row.id}`);
                console.log(`   Session: ${row.session_id}`);
                console.log(`   Content: "${row.content}"`);
                console.log(`   Question #: ${row.question_num}`);
                console.log(`   Category: ${row.category}`);
                console.log(`   Difficulty: ${row.difficulty}`);
                console.log(`   Source: ${row.source}`);
                console.log(`   Timestamp: ${row.timestamp}`);
            });
        }

        // Check if the specific question from the logs exists
        const specificQuestion = await client.query(`
            SELECT * FROM interview_messages 
            WHERE id = '5fad133f-3e92-412c-98b5-c8fd277989c2'
        `);

        console.log('\n🔍 Checking for the specific question from logs:');
        console.log('==============================================');
        if (specificQuestion.rows.length > 0) {
            const q = specificQuestion.rows[0];
            console.log('✅ Found the question from the logs!');
            console.log(`   ID: ${q.id}`);
            console.log(`   Content: "${q.content}"`);
            console.log(`   Session: ${q.session_id}`);
            console.log(`   Metadata: ${JSON.stringify(q.metadata, null, 2)}`);
            console.log(`   Timestamp: ${q.timestamp}`);
        } else {
            console.log('❌ Question from logs not found in database');
        }

        // Check total count
        const countResult = await client.query(`
            SELECT COUNT(*) as total_questions 
            FROM interview_messages 
            WHERE message_type = 'question'
        `);
        
        console.log(`\n📊 Total questions in database: ${countResult.rows[0].total_questions}`);

    } catch (err) {
        console.error('❌ Database error:', err.message);
    } finally {
        await client.end();
        console.log('\n🔌 Database connection closed');
    }
}

checkQuestions();
