-- Migration to add session_started_at field to sessions table
-- This field tracks when the interview session actually starts (as opposed to when it's created or connected)

-- Add the session_started_at column
ALTER TABLE sessions 
ADD COLUMN session_started_at TIMESTAMPTZ;

-- Add comment to explain the field
COMMENT ON COLUMN sessions.session_started_at IS 'Timestamp when the interview session actually started (first question asked or interview began)';

-- Optional: Create an index on session_started_at for performance if needed for queries
CREATE INDEX idx_sessions_started_at ON sessions(session_started_at);

-- Update any existing active sessions to have session_started_at = desktop_connected_at if they don't have it
UPDATE sessions 
SET session_started_at = desktop_connected_at 
WHERE status = 'active' 
  AND desktop_connected_at IS NOT NULL 
  AND session_started_at IS NULL;
