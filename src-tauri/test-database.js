// Test script to diagnose the database connectivity issue
// Run this in the browser console or add to your frontend code

async function testDatabaseDiagnostic() {
    console.log('🔍 Starting database diagnostic test...');
    
    try {
        // Test 1: Run database diagnostic
        console.log('\n=== Test 1: Database Diagnostic ===');
        const diagnostic = await window.safeInvoke('diagnose_database');
        console.log('📊 Database Diagnostic Result:', diagnostic);
        
        if (diagnostic.database_connected) {
            console.log('✅ Database is connected');
            console.log(`📋 Tables exist: ${diagnostic.tables_exist}`);
            if (diagnostic.sample_data_count !== null) {
                console.log(`🔢 Sessions count: ${diagnostic.sample_data_count}`);
            }
        } else {
            console.log('❌ Database connection failed');
            if (diagnostic.connection_error) {
                console.log(`🚨 Error: ${diagnostic.connection_error}`);
            }
        }
        
        // Test 2: Test with a sample session ID (if database is connected)
        if (diagnostic.database_connected) {
            console.log('\n=== Test 2: Session Query Test ===');
            
            // Try with a sample UUID (this will likely not exist, but should give us better error)
            const sampleSessionId = '123e4567-e89b-12d3-a456-426614174000';
            try {
                const sessionResult = await window.safeInvoke('test_session_query', sampleSessionId);
                console.log('✅ Session query result:', sessionResult);
            } catch (sessionError) {
                console.log('❌ Session query failed:', sessionError);
                
                // This should now give us better error information
                if (sessionError.includes('Session not found')) {
                    console.log('🔍 This is expected - the sample session ID does not exist');
                } else if (sessionError.includes('Multiple sessions found')) {
                    console.log('⚠️ Database consistency issue - multiple sessions with same ID');
                } else {
                    console.log('🚨 Unexpected error type:', sessionError);
                }
            }
        }
        
        console.log('\n=== Diagnostic Complete ===');
        
    } catch (error) {
        console.error('❌ Diagnostic test failed:', error);
    }
}

// Auto-run the test
testDatabaseDiagnostic();

// Also provide manual functions for testing
window.testDatabaseDiagnostic = testDatabaseDiagnostic;
window.testSpecificSession = async function(sessionId) {
    console.log(`🧪 Testing specific session: ${sessionId}`);
    try {
        const result = await window.safeInvoke('test_session_query', sessionId);
        console.log('✅ Result:', result);
    } catch (error) {
        console.log('❌ Error:', error);
    }
};

console.log('📝 Diagnostic functions loaded. You can also call:');
console.log('   - testDatabaseDiagnostic() - Run full diagnostic');
console.log('   - testSpecificSession("your-session-id") - Test a specific session ID');
