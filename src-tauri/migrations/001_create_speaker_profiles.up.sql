-- Migration: Create speaker profiles and related tables
-- Version: 001
-- Description: Initial schema for speaker identification and management

BEGIN TRANSACTION;

-- Speaker profiles table
CREATE TABLE IF NOT EXISTS speaker_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    identification_count INTEGER NOT NULL DEFAULT 0,
    confidence_threshold REAL NOT NULL DEFAULT 0.7,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    
    -- Voice characteristics (stored as JSON)
    pitch_range_min REAL,
    pitch_range_max REAL,
    pitch_mean REAL,
    speaking_rate REAL,
    quality_features TEXT, -- JSON blob
    gender TEXT,
    age_range_min INTEGER,
    age_range_max INTEGER,
    language_markers TEXT -- JSON array
);

-- Voice embeddings table (normalized for multiple embeddings per speaker)
CREATE TABLE IF NOT EXISTS voice_embeddings (
    id TEXT PRIMARY KEY,
    speaker_id TEXT NOT NULL,
    vector BLOB NOT NULL, -- Binary representation of float vector
    dimensions INTEGER NOT NULL,
    model_name TEXT NOT NULL,
    quality_score REAL NOT NULL,
    duration_seconds REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (speaker_id) REFERENCES speaker_profiles(id) ON DELETE CASCADE
);

-- Meeting speakers junction table
CREATE TABLE IF NOT EXISTS meeting_speakers (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL,
    speaker_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    speaking_time_seconds REAL NOT NULL DEFAULT 0.0,
    segment_count INTEGER NOT NULL DEFAULT 0,
    average_confidence REAL NOT NULL DEFAULT 0.0,
    first_spoken_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_spoken_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_verified BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT,
    
    FOREIGN KEY (speaker_id) REFERENCES speaker_profiles(id) ON DELETE CASCADE,
    UNIQUE(meeting_id, speaker_id) -- One record per speaker per meeting
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_speaker_profiles_name ON speaker_profiles(name);
CREATE INDEX IF NOT EXISTS idx_speaker_profiles_active ON speaker_profiles(is_active);
CREATE INDEX IF NOT EXISTS idx_speaker_profiles_updated ON speaker_profiles(updated_at);

CREATE INDEX IF NOT EXISTS idx_voice_embeddings_speaker ON voice_embeddings(speaker_id);
CREATE INDEX IF NOT EXISTS idx_voice_embeddings_model ON voice_embeddings(model_name);
CREATE INDEX IF NOT EXISTS idx_voice_embeddings_quality ON voice_embeddings(quality_score);

CREATE INDEX IF NOT EXISTS idx_meeting_speakers_meeting ON meeting_speakers(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_speakers_speaker ON meeting_speakers(speaker_id);
CREATE INDEX IF NOT EXISTS idx_meeting_speakers_time ON meeting_speakers(speaking_time_seconds);
CREATE INDEX IF NOT EXISTS idx_meeting_speakers_verified ON meeting_speakers(is_verified);

-- Triggers for auto-updating timestamps
CREATE TRIGGER IF NOT EXISTS speaker_profiles_updated_at
    AFTER UPDATE ON speaker_profiles
    BEGIN
        UPDATE speaker_profiles 
        SET updated_at = CURRENT_TIMESTAMP 
        WHERE id = NEW.id;
    END;

-- Comments for documentation (SQLite PRAGMA)
PRAGMA table_info(speaker_profiles);

COMMIT;