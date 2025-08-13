const { Client } = require('pg');

async function checkSpecificSession() {
    const targetSessionId = 'e302a575-1e13-4466-8ae7-7aea024df3ec';
    console.log('🔍 Checking for questions from specific session...');
    console.log(`📋 Session ID: ${targetSessionId}`);
    
    // Database connection
    const client = new Client({
        host: 'localhost',
        port: 5432,
        database: 'mockmate_db',
        user: 'mockmate_user',
        password: 'mockmate_2024!',
    });

    try {
        console.log('\n🔗 Connecting to database...');
        await client.connect();
        console.log('✅ Connected to database successfully!');

        // Check questions from the specific session
        console.log('\n📋 Checking questions from your session...');
        const sessionQuestions = await client.query(`
            SELECT 
                id,
                session_id,
                content as question_text,
                metadata,
                timestamp,
                EXTRACT(EPOCH FROM (NOW() - timestamp))/60 as minutes_ago
            FROM interview_messages 
            WHERE message_type = 'question' 
            AND session_id = $1
            ORDER BY timestamp DESC
        `, [targetSessionId]);

        if (sessionQuestions.rows.length === 0) {
            console.log('❌ No questions found for this session ID');
            console.log('\n🔍 Let me check if this session exists at all...');
            
            // Check if session exists in sessions table
            const sessionCheck = await client.query(`
                SELECT id, job_title, status, created_at
                FROM sessions 
                WHERE id = $1
            `, [targetSessionId]);
            
            if (sessionCheck.rows.length === 0) {
                console.log('❌ Session ID not found in sessions table either');
                console.log('💡 The session might not have been created or the ID is incorrect');
            } else {
                const session = sessionCheck.rows[0];
                console.log('✅ Session exists in sessions table:');
                console.log(`   Job Title: ${session.job_title}`);
                console.log(`   Status: ${session.status}`);
                console.log(`   Created: ${session.created_at}`);
                console.log('\n💡 Session exists but no questions have been stored yet');
            }
        } else {
            console.log(`✅ Found ${sessionQuestions.rows.length} question(s) for this session:`);
            console.log('-'.repeat(80));
            
            sessionQuestions.rows.forEach((row, i) => {
                console.log(`\n📝 Question #${i + 1}:`);
                console.log(`   ID: ${row.id}`);
                console.log(`   Text: "${row.question_text}"`);
                console.log(`   Timestamp: ${row.timestamp}`);
                console.log(`   Minutes ago: ${parseFloat(row.minutes_ago).toFixed(1)}`);
                
                if (row.metadata) {
                    try {
                        const meta = typeof row.metadata === 'string' ? JSON.parse(row.metadata) : row.metadata;
                        console.log(`   Source: ${meta.source || 'unknown'}`);
                        console.log(`   Category: ${meta.category || 'unknown'}`);
                        console.log(`   Difficulty: ${meta.difficulty || 'unknown'}`);
                        if (meta.timestamp) {
                            console.log(`   Created: ${meta.timestamp}`);
                        }
                    } catch (e) {
                        console.log(`   Metadata: ${row.metadata}`);
                    }
                }
            });
        }

        // Check answers from this session too
        console.log('\n💬 Checking answers from your session...');
        const sessionAnswers = await client.query(`
            SELECT 
                id,
                parent_message_id,
                content as answer_text,
                metadata,
                timestamp
            FROM interview_messages 
            WHERE message_type = 'answer' 
            AND session_id = $1
            ORDER BY timestamp DESC
        `, [targetSessionId]);

        if (sessionAnswers.rows.length === 0) {
            console.log('❌ No answers found for this session ID');
        } else {
            console.log(`✅ Found ${sessionAnswers.rows.length} answer(s) for this session:`);
            sessionAnswers.rows.forEach((row, i) => {
                console.log(`\n💬 Answer #${i + 1}:`);
                console.log(`   ID: ${row.id}`);
                console.log(`   Question ID: ${row.parent_message_id}`);
                console.log(`   Text: "${row.answer_text.substring(0, 100)}${row.answer_text.length > 100 ? '...' : ''}"`);
                console.log(`   Timestamp: ${row.timestamp}`);
            });
        }

        // Check for any recent questions from any session (to compare)
        console.log('\n📋 Recent questions from ALL sessions (for comparison):');
        const allRecentQuestions = await client.query(`
            SELECT 
                id,
                session_id,
                LEFT(content, 60) as question_text,
                metadata->>'source' as source,
                timestamp,
                EXTRACT(EPOCH FROM (NOW() - timestamp))/60 as minutes_ago
            FROM interview_messages 
            WHERE message_type = 'question' 
            AND timestamp >= NOW() - INTERVAL '2 hours'
            ORDER BY timestamp DESC
            LIMIT 5
        `);

        if (allRecentQuestions.rows.length > 0) {
            allRecentQuestions.rows.forEach((row, i) => {
                const isTargetSession = row.session_id === targetSessionId;
                const marker = isTargetSession ? '🎯' : '📝';
                console.log(`${marker} ${row.question_text} (${parseFloat(row.minutes_ago).toFixed(1)}m ago) ${isTargetSession ? '← YOUR SESSION' : ''}`);
                console.log(`   Session: ${row.session_id}`);
                console.log(`   Source: ${row.source || 'unknown'}`);
            });
        }

    } catch (error) {
        console.error('❌ Database error:', error.message);
    } finally {
        await client.end();
        console.log('\n🔒 Database connection closed');
    }
}

// Run the check
checkSpecificSession().catch(console.error);
