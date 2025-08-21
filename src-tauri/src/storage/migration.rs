use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::task;
use std::sync::Arc;

use crate::storage::Database;

/// Database migration management
pub struct MigrationManager {
    db: Database,
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: &'static str,
    pub down_sql: &'static str,
}

impl MigrationManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all available migrations
    pub fn get_migrations() -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                name: "create_speaker_profiles".to_string(),
                up_sql: include_str!("../../migrations/001_create_speaker_profiles.up.sql"),
                down_sql: include_str!("../../migrations/001_create_speaker_profiles.down.sql"),
            },
        ]
    }

    /// Run all pending migrations
    pub async fn migrate_up(&self) -> Result<Vec<u32>> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<Vec<u32>> {
            let conn = connection.lock().unwrap();
            
            // Create migrations table if it doesn't exist
            conn.execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    checksum TEXT NOT NULL
                );",
                [],
            ).context("Failed to create migrations table")?;

            // Get applied migrations
            let mut applied_migrations = HashMap::new();
            {
                let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
                let rows = stmt.query_map([], |row| {
                    Ok(row.get::<_, i64>(0)? as u32)
                })?;

                for version in rows {
                    applied_migrations.insert(version?, true);
                }
            }

            let mut applied = Vec::new();
            let migrations = Self::get_migrations();

            // Apply pending migrations
            for migration in migrations {
                if !applied_migrations.contains_key(&migration.version) {
                    tracing::info!("Applying migration {} - {}", migration.version, migration.name);
                    
                    // Calculate checksum for migration
                    let checksum = format!("{:x}", md5::compute(migration.up_sql.as_bytes()));
                    
                    // Execute migration in a transaction
                    let tx = conn.unchecked_transaction()?;
                    
                    // Run the migration SQL
                    tx.execute_batch(migration.up_sql)
                        .with_context(|| format!("Failed to execute migration {}", migration.version))?;
                    
                    // Record the migration
                    tx.execute(
                        "INSERT INTO schema_migrations (version, name, applied_at, checksum) 
                         VALUES (?1, ?2, CURRENT_TIMESTAMP, ?3)",
                        [
                            &migration.version.to_string(),
                            &migration.name,
                            &checksum,
                        ],
                    ).with_context(|| format!("Failed to record migration {}", migration.version))?;
                    
                    tx.commit().with_context(|| format!("Failed to commit migration {}", migration.version))?;
                    
                    applied.push(migration.version);
                    tracing::info!("Successfully applied migration {}", migration.version);
                }
            }

            Ok(applied)
        }).await?
    }

    /// Rollback migrations to a specific version
    pub async fn migrate_down(&self, target_version: u32) -> Result<Vec<u32>> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<Vec<u32>> {
            let conn = connection.lock().unwrap();
            
            // Get applied migrations in reverse order
            let mut stmt = conn.prepare(
                "SELECT version, name FROM schema_migrations 
                 WHERE version > ?1 
                 ORDER BY version DESC"
            )?;
            
            let migration_rows = stmt.query_map([target_version], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, String>(1)?,
                ))
            })?;

            let mut applied_migrations: Vec<(u32, String)> = Vec::new();
            for row in migration_rows {
                applied_migrations.push(row?);
            }

            let all_migrations = Self::get_migrations();
            let mut migration_map = HashMap::new();
            for migration in all_migrations {
                migration_map.insert(migration.version, migration);
            }

            let mut rolled_back = Vec::new();

            // Rollback migrations in reverse order
            for (version, name) in applied_migrations {
                if let Some(migration) = migration_map.get(&version) {
                    tracing::info!("Rolling back migration {} - {}", version, name);
                    
                    // Execute rollback in a transaction
                    let tx = conn.unchecked_transaction()?;
                    
                    // Run the rollback SQL
                    tx.execute_batch(migration.down_sql)
                        .with_context(|| format!("Failed to rollback migration {}", version))?;
                    
                    // Remove migration record
                    tx.execute(
                        "DELETE FROM schema_migrations WHERE version = ?1",
                        [&version.to_string()],
                    ).with_context(|| format!("Failed to remove migration record {}", version))?;
                    
                    tx.commit().with_context(|| format!("Failed to commit rollback {}", version))?;
                    
                    rolled_back.push(version);
                    tracing::info!("Successfully rolled back migration {}", version);
                } else {
                    tracing::warn!("No rollback found for migration {}", version);
                }
            }

            Ok(rolled_back)
        }).await?
    }

    /// Get current database version
    pub async fn get_current_version(&self) -> Result<u32> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<u32> {
            let conn = connection.lock().unwrap();
            
            // Check if migrations table exists
            let table_exists: bool = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations'")?
                .exists([])?;
            
            if !table_exists {
                return Ok(0);
            }

            // Get highest applied migration version
            let mut stmt = conn.prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")?;
            let version: i64 = stmt.query_row([], |row| row.get(0))?;
            
            Ok(version as u32)
        }).await?
    }

    /// Get migration history
    pub async fn get_migration_history(&self) -> Result<Vec<MigrationRecord>> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<Vec<MigrationRecord>> {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT version, name, applied_at, checksum FROM schema_migrations ORDER BY version"
            )?;
            
            let records = stmt.query_map([], |row| {
                Ok(MigrationRecord {
                    version: row.get::<_, i64>(0)? as u32,
                    name: row.get(1)?,
                    applied_at: row.get(2)?,
                    checksum: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(records)
        }).await?
    }
}

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: u32,
    pub name: String,
    pub applied_at: String,
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> Database {
        let temp_file = NamedTempFile::new().unwrap();
        Database::new(temp_file.path()).await.unwrap()
    }

    #[tokio::test]
    async fn test_migration_up() {
        let db = create_test_database().await;
        let manager = MigrationManager::new(db);

        let applied = manager.migrate_up().await.unwrap();
        assert!(!applied.is_empty());

        let version = manager.get_current_version().await.unwrap();
        assert!(version > 0);
    }

    #[tokio::test]
    async fn test_migration_history() {
        let db = create_test_database().await;
        let manager = MigrationManager::new(db);

        // Apply migrations
        manager.migrate_up().await.unwrap();

        // Check history
        let history = manager.get_migration_history().await.unwrap();
        assert!(!history.is_empty());
        
        let first_migration = &history[0];
        assert_eq!(first_migration.version, 1);
        assert_eq!(first_migration.name, "create_speaker_profiles");
    }

    #[tokio::test]
    async fn test_migration_down() {
        let db = create_test_database().await;
        let manager = MigrationManager::new(db);

        // Apply migrations
        let applied = manager.migrate_up().await.unwrap();
        assert!(!applied.is_empty());

        // Rollback migrations
        let rolled_back = manager.migrate_down(0).await.unwrap();
        assert_eq!(applied.len(), rolled_back.len());

        let version = manager.get_current_version().await.unwrap();
        assert_eq!(version, 0);
    }
}

// Add MD5 dependency to Cargo.toml for checksum calculation
// This is a simple implementation - in production you might want to use a more robust approach