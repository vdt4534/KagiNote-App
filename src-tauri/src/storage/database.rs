use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task;
use uuid::Uuid;

/// Database connection manager for speaker profiles
#[derive(Clone)]
pub struct Database {
    pub connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path = db_path.as_ref().to_path_buf();
        
        let connection = task::spawn_blocking(move || -> Result<Connection> {
            let conn = Connection::open_with_flags(
                &path,
                OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
            ).context("Failed to open SQLite database")?;
            
            // Enable foreign key constraints
            conn.execute("PRAGMA foreign_keys = ON;", [])
                .context("Failed to enable foreign keys")?;
                
            // Set WAL mode for better concurrency
            conn.execute("PRAGMA journal_mode = WAL;", [])
                .context("Failed to set WAL mode")?;
                
            // Optimize for performance
            conn.execute("PRAGMA synchronous = NORMAL;", [])
                .context("Failed to set synchronous mode")?;
                
            conn.execute("PRAGMA cache_size = -2000;", [])
                .context("Failed to set cache size")?;
                
            Ok(conn)
        }).await??;
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    
    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        let connection = Arc::clone(&self.connection);
        
        task::spawn_blocking(move || -> Result<()> {
            let conn = connection.lock().unwrap();
            
            // Read and execute migration file
            let migration_sql = include_str!("../../migrations/001_create_speaker_profiles.up.sql");
            conn.execute_batch(migration_sql)
                .context("Failed to execute migration")?;
                
            Ok(())
        }).await?
    }
    
    /// Execute a query with parameters
    pub async fn execute<P>(&self, sql: &str, params: P) -> Result<usize>
    where
        P: rusqlite::Params + Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        let sql = sql.to_string();
        
        task::spawn_blocking(move || -> Result<usize> {
            let conn = connection.lock().unwrap();
            let rows_affected = conn.execute(&sql, params)
                .context("Failed to execute SQL")?;
            Ok(rows_affected)
        }).await?
    }
    
    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<()> {
        self.execute("BEGIN TRANSACTION;", []).await?;
        Ok(())
    }
    
    /// Commit a transaction
    pub async fn commit_transaction(&self) -> Result<()> {
        self.execute("COMMIT;", []).await?;
        Ok(())
    }
    
    /// Rollback a transaction
    pub async fn rollback_transaction(&self) -> Result<()> {
        self.execute("ROLLBACK;", []).await?;
        Ok(())
    }
    
    /// Check if the database is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let connection = Arc::clone(&self.connection);
        
        task::spawn_blocking(move || -> Result<bool> {
            let conn = connection.lock().unwrap();
            let mut stmt = conn.prepare("SELECT 1;")?;
            let result: i32 = stmt.query_row([], |row| row.get(0))?;
            Ok(result == 1)
        }).await?
    }
}

/// Convert Vec<f32> to BLOB for storage
pub fn vector_to_blob(vector: &[f32]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(vector.len() * 4);
    for &value in vector {
        blob.extend_from_slice(&value.to_le_bytes());
    }
    blob
}

/// Convert BLOB back to Vec<f32>
pub fn blob_to_vector(blob: &[u8]) -> Result<Vec<f32>> {
    if blob.len() % 4 != 0 {
        anyhow::bail!("Invalid blob size for float vector");
    }
    
    let mut vector = Vec::with_capacity(blob.len() / 4);
    for chunk in blob.chunks_exact(4) {
        let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
        vector.push(f32::from_le_bytes(bytes));
    }
    
    Ok(vector)
}

/// Convert UUID to string for SQLite storage
pub fn uuid_to_string(uuid: &Uuid) -> String {
    uuid.to_string()
}

/// Convert string back to UUID
pub fn string_to_uuid(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).context("Invalid UUID string")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let is_healthy = db.health_check().await.unwrap();
        assert!(is_healthy);
    }
    
    #[tokio::test]
    async fn test_database_migration() -> Result<()> {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Run migration
        db.migrate().await.unwrap();
        
        // Verify tables exist by querying them
        let connection = Arc::clone(&db.connection);
        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = connection.lock().unwrap();
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table';")?;
            let table_names: Result<Vec<String>, _> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect();
            
            let tables = table_names?;
            assert!(tables.contains(&"speaker_profiles".to_string()));
            assert!(tables.contains(&"voice_embeddings".to_string()));
            assert!(tables.contains(&"meeting_speakers".to_string()));
            
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    #[test]
    fn test_vector_blob_conversion() {
        let original = vec![1.0, 2.5, -3.7, 0.0, 100.1];
        let blob = vector_to_blob(&original);
        let restored = blob_to_vector(&blob).unwrap();
        
        assert_eq!(original.len(), restored.len());
        for (a, b) in original.iter().zip(restored.iter()) {
            assert!((a - b).abs() < 0.001);
        }
    }
    
    #[test]
    fn test_uuid_string_conversion() {
        let original = Uuid::new_v4();
        let string = uuid_to_string(&original);
        let restored = string_to_uuid(&string).unwrap();
        
        assert_eq!(original, restored);
    }
}