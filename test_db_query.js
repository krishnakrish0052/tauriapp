const { Client } = require('pg');

async function checkStoredQuestions() {
    console.log('🔍 Checking for stored questions in MockMate database...');
    
    // Database connection (using your .env file credentials)
    const client = new Client({
        host: 'localhost',
        port: 5432,
        database: 'mockmate_db',
        user: 'mockmate_user',
        password: 'mockmate_2024!',
    });

    try {
        console.log('🔗 Connecting to database...');
        await client.connect();
        console.log('✅ Connected to database successfully!');

        // Check if interview_messages table exists
        const tableCheck = await client.query(`
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = 'interview_messages'
            );
        `);

        if (!tableCheck.rows[0].exists) {
            console.log('❌ interview_messages table does not exist');
            return;
        }

        console.log('✅ interview_messages table found');

        // Get recent questions (last 24 hours)
        console.log('\n📋 Checking recent questions (last 24 hours)...');
        const recentQuestions = await client.query(`
            SELECT 
                id,
                session_id,
                content as question_text,
                metadata,
                timestamp,
                EXTRACT(EPOCH FROM (NOW() - timestamp))/3600 as hours_ago
            FROM interview_messages 
            WHERE message_type = 'question' 
            AND timestamp >= NOW() - INTERVAL '24 hours'
            ORDER BY timestamp DESC
        `);

        if (recentQuestions.rows.length === 0) {
            console.log('❌ No questions found in the last 24 hours');
        } else {
            console.log(`✅ Found ${recentQuestions.rows.length} recent question(s):`);
            console.log('-'.repeat(80));
            
            recentQuestions.rows.forEach((row, i) => {
                console.log(`\n📝 Question #${i + 1}:`);
                console.log(`   ID: ${row.id}`);
                console.log(`   Session: ${row.session_id}`);
                console.log(`   Text: ${row.question_text.substring(0, 100)}${row.question_text.length > 100 ? '...' : ''}`);
                console.log(`   Timestamp: ${row.timestamp}`);
                console.log(`   Hours ago: ${parseFloat(row.hours_ago).toFixed(1)}`);
                
                if (row.metadata) {
                    const meta = typeof row.metadata === 'string' ? JSON.parse(row.metadata) : row.metadata;
                    console.log(`   Source: ${meta.source || 'unknown'}`);
                    console.log(`   Category: ${meta.category || 'unknown'}`);
                    console.log(`   Difficulty: ${meta.difficulty || 'unknown'}`);
                }
            });
        }

        // Get all questions if no recent ones
        if (recentQuestions.rows.length === 0) {
            console.log('\n📋 Checking all questions (most recent 10)...');
            const allQuestions = await client.query(`
                SELECT 
                    id,
                    session_id,
                    content as question_text,
                    metadata,
                    timestamp
                FROM interview_messages 
                WHERE message_type = 'question'
                ORDER BY timestamp DESC
                LIMIT 10
            `);

            if (allQuestions.rows.length === 0) {
                console.log('❌ No questions found in the database at all');
            } else {
                console.log(`✅ Found ${allQuestions.rows.length} total question(s):`);
                console.log('-'.repeat(80));
                
                allQuestions.rows.forEach((row, i) => {
                    console.log(`\n📝 Question #${i + 1}:`);
                    console.log(`   ID: ${row.id}`);
                    console.log(`   Session: ${row.session_id}`);
                    console.log(`   Text: ${row.question_text}`);
                    console.log(`   Timestamp: ${row.timestamp}`);
                    
                    if (row.metadata) {
                        const meta = typeof row.metadata === 'string' ? JSON.parse(row.metadata) : row.metadata;
                        console.log(`   Source: ${meta.source || 'unknown'}`);
                        console.log(`   Category: ${meta.category || 'unknown'}`);
                        console.log(`   Difficulty: ${meta.difficulty || 'unknown'}`);
                    }
                });
            }
        }

        // Get summary stats
        console.log('\n📊 Database Summary:');
        const summary = await client.query(`
            SELECT 
                (SELECT COUNT(*) FROM interview_messages WHERE message_type = 'question') as total_questions,
                (SELECT COUNT(*) FROM interview_messages WHERE message_type = 'answer') as total_answers,
                (SELECT COUNT(DISTINCT session_id) FROM interview_messages) as unique_sessions
        `);

        const stats = summary.rows[0];
        console.log(`   Total Questions: ${stats.total_questions}`);
        console.log(`   Total Answers: ${stats.total_answers}`);
        console.log(`   Unique Sessions: ${stats.unique_sessions}`);

    } catch (error) {
        console.error('❌ Database error:', error.message);
        
        if (error.code === 'ECONNREFUSED') {
            console.log('\n💡 Make sure PostgreSQL server is running and accepting connections');
        } else if (error.code === '28P01') {
            console.log('\n💡 Authentication failed - check your database credentials');
        } else if (error.code === '3D000') {
            console.log('\n💡 Database "mockmate_db" does not exist');
        }
    } finally {
        await client.end();
        console.log('\n🔒 Database connection closed');
    }
}

// Run the check
checkStoredQuestions().catch(console.error);
