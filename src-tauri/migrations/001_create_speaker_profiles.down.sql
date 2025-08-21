-- Rollback migration: Drop speaker profiles and related tables
-- Version: 001
-- Description: Clean rollback of speaker identification schema

BEGIN TRANSACTION;

-- Drop triggers first
DROP TRIGGER IF EXISTS speaker_profiles_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_meeting_speakers_verified;
DROP INDEX IF EXISTS idx_meeting_speakers_time;
DROP INDEX IF EXISTS idx_meeting_speakers_speaker;
DROP INDEX IF EXISTS idx_meeting_speakers_meeting;

DROP INDEX IF EXISTS idx_voice_embeddings_quality;
DROP INDEX IF EXISTS idx_voice_embeddings_model;
DROP INDEX IF EXISTS idx_voice_embeddings_speaker;

DROP INDEX IF EXISTS idx_speaker_profiles_updated;
DROP INDEX IF EXISTS idx_speaker_profiles_active;
DROP INDEX IF EXISTS idx_speaker_profiles_name;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS meeting_speakers;
DROP TABLE IF EXISTS voice_embeddings;
DROP TABLE IF EXISTS speaker_profiles;

COMMIT;