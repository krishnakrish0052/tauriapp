const { Client } = require('pg');

async function checkAnswers() {
    const client = new Client({
        host: 'localhost',
        port: 5432,
        database: 'mockmate_db',
        user: 'mockmate_user',
        password: 'mockmate_2024!',
    });

    try {
        await client.connect();
        console.log('✅ Connected to database');

        // Check recent answers
        const result = await client.query(`
            SELECT 
                id, 
                parent_message_id as question_id,
                session_id, 
                message_type, 
                content, 
                metadata->>'source' as source,
                metadata->>'aiProvider' as ai_provider,
                metadata->>'aiModel' as ai_model,
                metadata->>'responseTime' as response_time,
                timestamp 
            FROM interview_messages 
            WHERE message_type = 'answer' 
            ORDER BY timestamp DESC 
            LIMIT 5
        `);

        console.log('\n💬 Recent Answers in Database:');
        console.log('==============================');
        
        if (result.rows.length === 0) {
            console.log('❌ No answers found in database');
        } else {
            result.rows.forEach((row, index) => {
                console.log(`\n${index + 1}. Answer ID: ${row.id}`);
                console.log(`   Question ID: ${row.question_id}`);
                console.log(`   Session: ${row.session_id}`);
                console.log(`   Content: "${row.content.substring(0, 100)}..."`);
                console.log(`   Source: ${row.source}`);
                console.log(`   AI Provider: ${row.ai_provider}`);
                console.log(`   AI Model: ${row.ai_model}`);
                console.log(`   Response Time: ${row.response_time}`);
                console.log(`   Timestamp: ${row.timestamp}`);
            });
        }

        // Check for answers linked to the specific question from your test
        const linkedAnswers = await client.query(`
            SELECT * FROM interview_messages 
            WHERE parent_message_id = '5fad133f-3e92-412c-98b5-c8fd277989c2'
            AND message_type = 'answer'
        `);

        console.log('\n🔗 Answers linked to your "what is docker?" question:');
        console.log('====================================================');
        if (linkedAnswers.rows.length > 0) {
            linkedAnswers.rows.forEach((answer) => {
                console.log('✅ Found linked answer!');
                console.log(`   Answer ID: ${answer.id}`);
                console.log(`   Content length: ${answer.content.length} characters`);
                console.log(`   Metadata: ${JSON.stringify(answer.metadata, null, 2)}`);
                console.log(`   Timestamp: ${answer.timestamp}`);
            });
        } else {
            console.log('❌ No answers found for your test question');
        }

        // Check total count of answers
        const countResult = await client.query(`
            SELECT COUNT(*) as total_answers 
            FROM interview_messages 
            WHERE message_type = 'answer'
        `);
        
        console.log(`\n📊 Total answers in database: ${countResult.rows[0].total_answers}`);

        // Check question-answer pairs for the session
        const qaPairs = await client.query(`
            SELECT 
                q.content as question,
                a.content as answer,
                q.timestamp as question_time,
                a.timestamp as answer_time,
                a.metadata->>'aiProvider' as provider,
                a.metadata->>'aiModel' as model
            FROM interview_messages q
            LEFT JOIN interview_messages a ON q.id = a.parent_message_id
            WHERE q.session_id = 'e302a575-1e13-4466-8ae7-7aea024df3ec'
            AND q.message_type = 'question'
            ORDER BY q.timestamp DESC
        `);

        console.log('\n🔄 Question-Answer Pairs for your session:');
        console.log('=========================================');
        qaPairs.rows.forEach((pair, index) => {
            console.log(`\n${index + 1}. Q: "${pair.question}"`);
            if (pair.answer) {
                console.log(`   A: "${pair.answer.substring(0, 150)}..."`);
                console.log(`   Provider: ${pair.provider}, Model: ${pair.model}`);
                console.log(`   Time gap: ${new Date(pair.answer_time) - new Date(pair.question_time)}ms`);
            } else {
                console.log(`   A: ❌ No answer found`);
            }
        });

    } catch (err) {
        console.error('❌ Database error:', err.message);
    } finally {
        await client.end();
        console.log('\n🔌 Database connection closed');
    }
}

checkAnswers();
