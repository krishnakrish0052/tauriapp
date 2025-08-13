# MockMate Database Question Checker
# PowerShell script to check if questions are stored in the database

Write-Host "🔍 MockMate Database Question Checker" -ForegroundColor Cyan
Write-Host "=" * 50

# Database connection parameters (same defaults as your Rust code)
$env:PGHOST = if ($env:DB_HOST) { $env:DB_HOST } else { "localhost" }
$env:PGPORT = if ($env:DB_PORT) { $env:DB_PORT } else { "5432" }
$env:PGDATABASE = if ($env:DB_NAME) { $env:DB_NAME } else { "mockmate_db" }
$env:PGUSER = if ($env:DB_USER) { $env:DB_USER } else { "mockmate_user" }
$env:PGPASSWORD = if ($env:DB_PASSWORD) { $env:DB_PASSWORD } else { "" }

Write-Host "🔗 Connecting to database: $env:PGUSER@$env:PGHOST`:$env:PGPORT/$env:PGDATABASE" -ForegroundColor Yellow

# Check if psql is available
$psqlPath = Get-Command psql -ErrorAction SilentlyContinue
if (-not $psqlPath) {
    Write-Host "❌ PostgreSQL client (psql) not found in PATH" -ForegroundColor Red
    Write-Host "Please install PostgreSQL client tools or add psql to your PATH" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Found PostgreSQL client: $($psqlPath.Source)" -ForegroundColor Green

# Test database connection
Write-Host "`n1. Testing database connection..."
$connectionTest = & psql -c "SELECT version();" -t 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Database connection failed:" -ForegroundColor Red
    Write-Host $connectionTest -ForegroundColor Red
    Write-Host "`n💡 Make sure:" -ForegroundColor Yellow
    Write-Host "   - PostgreSQL server is running" -ForegroundColor Yellow
    Write-Host "   - Database 'mockmate_db' exists" -ForegroundColor Yellow
    Write-Host "   - User 'mockmate_user' has access" -ForegroundColor Yellow
    Write-Host "   - Correct credentials in environment variables" -ForegroundColor Yellow
    exit 1
}
Write-Host "✅ Database connection successful!" -ForegroundColor Green

# Check if interview_messages table exists
Write-Host "`n2. Checking table structure..."
$tableCheck = & psql -c "\d interview_messages" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ interview_messages table not found" -ForegroundColor Red
    Write-Host $tableCheck -ForegroundColor Red
    exit 1
}
Write-Host "✅ interview_messages table found" -ForegroundColor Green

# Check recent questions (last 24 hours)
Write-Host "`n3. Checking recent questions (last 24 hours)..."
$recentQuestionsQuery = @"
SELECT 
    id,
    session_id,
    LEFT(content, 100) as question_text,
    metadata->>'source' as source,
    metadata->>'category' as category,
    timestamp,
    EXTRACT(EPOCH FROM (NOW() - timestamp))/3600 as hours_ago
FROM interview_messages 
WHERE message_type = 'question' 
AND timestamp >= NOW() - INTERVAL '24 hours'
ORDER BY timestamp DESC;
"@

$recentQuestions = & psql -c $recentQuestionsQuery 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error querying recent questions:" -ForegroundColor Red
    Write-Host $recentQuestions -ForegroundColor Red
} else {
    Write-Host $recentQuestions -ForegroundColor White
}

# Check all questions (last 10)
Write-Host "`n4. Checking all questions (most recent 10)..."
$allQuestionsQuery = @"
SELECT 
    id,
    session_id,
    LEFT(content, 80) as question_text,
    metadata->>'source' as source,
    metadata->>'category' as category,
    timestamp
FROM interview_messages 
WHERE message_type = 'question'
ORDER BY timestamp DESC
LIMIT 10;
"@

$allQuestions = & psql -c $allQuestionsQuery 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error querying all questions:" -ForegroundColor Red
    Write-Host $allQuestions -ForegroundColor Red
} else {
    Write-Host $allQuestions -ForegroundColor White
}

# Summary counts
Write-Host "`n5. Database summary..."
$summaryQuery = @"
SELECT 
    (SELECT COUNT(*) FROM interview_messages WHERE message_type = 'question') as total_questions,
    (SELECT COUNT(*) FROM interview_messages WHERE message_type = 'answer') as total_answers,
    (SELECT COUNT(DISTINCT session_id) FROM interview_messages) as unique_sessions;
"@

$summary = & psql -c $summaryQuery 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error getting summary:" -ForegroundColor Red
    Write-Host $summary -ForegroundColor Red
} else {
    Write-Host $summary -ForegroundColor Cyan
}

Write-Host "`nDatabase check complete!" -ForegroundColor Green
