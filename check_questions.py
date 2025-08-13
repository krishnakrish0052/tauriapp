#!/usr/bin/env python3
"""
Database Query Tool to Check Stored Questions
Checks the interview_messages table for recently stored questions
"""

import psycopg2
import json
from datetime import datetime, timedelta
import os

def connect_to_database():
    """Connect to PostgreSQL database using environment variables or defaults"""
    try:
        # Use environment variables or defaults (same as your Rust code)
        host = os.getenv('DB_HOST', 'localhost')
        port = int(os.getenv('DB_PORT', '5432'))
        dbname = os.getenv('DB_NAME', 'mockmate_db')
        user = os.getenv('DB_USER', 'mockmate_user')
        password = os.getenv('DB_PASSWORD', '')
        
        print(f"🔗 Connecting to database: {user}@{host}:{port}/{dbname}")
        
        conn = psycopg2.connect(
            host=host,
            port=port,
            database=dbname,
            user=user,
            password=password
        )
        
        print("✅ Database connection successful!")
        return conn
        
    except Exception as e:
        print(f"❌ Database connection failed: {e}")
        return None

def check_recent_questions(conn, hours_back=24):
    """Check for questions stored in the last X hours"""
    try:
        cursor = conn.cursor()
        
        # Calculate time threshold
        time_threshold = datetime.utcnow() - timedelta(hours=hours_back)
        
        query = """
        SELECT 
            id,
            session_id,
            content as question_text,
            metadata,
            timestamp,
            EXTRACT(EPOCH FROM (NOW() - timestamp))/3600 as hours_ago
        FROM interview_messages 
        WHERE message_type = 'question' 
        AND timestamp >= %s
        ORDER BY timestamp DESC
        """
        
        cursor.execute(query, (time_threshold,))
        questions = cursor.fetchall()
        
        if not questions:
            print(f"❌ No questions found in the last {hours_back} hours")
            return []
            
        print(f"✅ Found {len(questions)} question(s) in the last {hours_back} hours:")
        print("-" * 80)
        
        for i, (id, session_id, question_text, metadata, timestamp, hours_ago) in enumerate(questions, 1):
            print(f"\n📝 Question #{i}:")
            print(f"   ID: {id}")
            print(f"   Session: {session_id}")
            print(f"   Text: {question_text[:100]}{'...' if len(question_text) > 100 else ''}")
            print(f"   Timestamp: {timestamp}")
            print(f"   Time ago: {hours_ago:.1f} hours")
            
            if metadata:
                try:
                    meta = json.loads(metadata) if isinstance(metadata, str) else metadata
                    print(f"   Source: {meta.get('source', 'unknown')}")
                    print(f"   Category: {meta.get('category', 'unknown')}")
                    print(f"   Difficulty: {meta.get('difficulty', 'unknown')}")
                except:
                    print(f"   Metadata: {metadata}")
            
        return questions
        
    except Exception as e:
        print(f"❌ Error querying questions: {e}")
        return []

def check_all_questions(conn, limit=10):
    """Check all questions in the database (most recent first)"""
    try:
        cursor = conn.cursor()
        
        query = """
        SELECT 
            id,
            session_id,
            content as question_text,
            metadata,
            timestamp
        FROM interview_messages 
        WHERE message_type = 'question'
        ORDER BY timestamp DESC
        LIMIT %s
        """
        
        cursor.execute(query, (limit,))
        questions = cursor.fetchall()
        
        if not questions:
            print("❌ No questions found in the database")
            return []
            
        print(f"✅ Found {len(questions)} most recent question(s):")
        print("-" * 80)
        
        for i, (id, session_id, question_text, metadata, timestamp) in enumerate(questions, 1):
            print(f"\n📝 Question #{i}:")
            print(f"   ID: {id}")
            print(f"   Session: {session_id}")
            print(f"   Text: {question_text}")
            print(f"   Timestamp: {timestamp}")
            
            if metadata:
                try:
                    meta = json.loads(metadata) if isinstance(metadata, str) else metadata
                    print(f"   Source: {meta.get('source', 'unknown')}")
                    print(f"   Category: {meta.get('category', 'unknown')}")
                    print(f"   Difficulty: {meta.get('difficulty', 'unknown')}")
                except:
                    print(f"   Metadata: {metadata}")
            
        return questions
        
    except Exception as e:
        print(f"❌ Error querying questions: {e}")
        return []

def check_table_structure(conn):
    """Check the structure of interview_messages table"""
    try:
        cursor = conn.cursor()
        
        query = """
        SELECT column_name, data_type, is_nullable, column_default
        FROM information_schema.columns 
        WHERE table_name = 'interview_messages'
        ORDER BY ordinal_position
        """
        
        cursor.execute(query)
        columns = cursor.fetchall()
        
        if not columns:
            print("❌ interview_messages table not found")
            return
            
        print("📋 interview_messages table structure:")
        print("-" * 60)
        for col_name, data_type, nullable, default in columns:
            print(f"   {col_name:20} | {data_type:15} | {'NULL' if nullable == 'YES' else 'NOT NULL':8} | {default or ''}")
        
    except Exception as e:
        print(f"❌ Error checking table structure: {e}")

def main():
    print("🔍 MockMate Database Question Checker")
    print("=" * 50)
    
    # Connect to database
    conn = connect_to_database()
    if not conn:
        return
    
    try:
        # Check table structure
        print("\n1. Checking table structure...")
        check_table_structure(conn)
        
        # Check recent questions (last 24 hours)
        print("\n2. Checking recent questions (last 24 hours)...")
        recent_questions = check_recent_questions(conn, 24)
        
        # If no recent questions, check all questions
        if not recent_questions:
            print("\n3. Checking all questions (last 10)...")
            check_all_questions(conn, 10)
        
        # Check if there are any questions at all
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM interview_messages WHERE message_type = 'question'")
        total_questions = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM interview_messages WHERE message_type = 'answer'")
        total_answers = cursor.fetchone()[0]
        
        print(f"\n📊 Database Summary:")
        print(f"   Total Questions: {total_questions}")
        print(f"   Total Answers: {total_answers}")
        
    finally:
        conn.close()
        print("\n🔒 Database connection closed")

if __name__ == "__main__":
    main()
