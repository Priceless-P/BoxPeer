use base64::decode;
use futures::{StreamExt, TryFutureExt};
use libp2p::PeerId;
use sha2::{Digest, Sha256};
use sqlx::migrate::MigrateDatabase;
use sqlx::pool::PoolOptions;
use sqlx::{query, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use tauri::api::path::{cache_dir, data_dir};
use tokio::fs;
use tokio::io::AsyncReadExt;

/// Manages content and its metadata using a SQLite database.
#[derive(Debug, Clone)]
pub struct ContentManager {
    pool: SqlitePool,
}

/// Metadata for a file.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub content_hash: String,
    pub file_name: String,
    pub file_size: u64,
    pub chunk_count: i64,
    pub peers: Vec<String>,
}

impl ContentManager {
    /// Creates a new `ContentManager` with a connection pool to the SQLite database.
    ///
    /// # Arguments
    ///
    /// * `database_url` - The URL of the SQLite database (e.g., "sqlite://content_manager.db").
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ContentManager` or an error message.
    pub async fn new() -> Result<Self, String> {
        let data_directory = data_dir().ok_or("Could not determine data directory")?;
        let db_path = "content_manager.db";
        let db_path_full = data_directory.join(db_path);

        // Debug output
        println!("Data directory: {:?}", data_directory);
        println!("Database path: {:?}", db_path_full);

        // Ensure the directory exists
        if !data_directory.exists() {
            fs::create_dir_all(&data_directory)
                .map_err(|e| e.to_string())
                .await?;
        }

        // Create an empty file to ensure it's writable
        fs::File::create(&db_path_full)
            .map_err(|e| e.to_string())
            .await?;
        let db_url = format!("sqlite://{}", db_path_full.to_string_lossy());

        //let db_url = String::from("sqlite://sqlite.db");
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await.unwrap();
        }
        let pool = PoolOptions::new()
            .max_connections(6)
            .connect(&*db_url)
            .await
            .unwrap();

        // Initialize the `provided_contents` table.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS provided_contents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash TEXT UNIQUE,
                file_name TEXT,
                chunk_count INTEGER,
                peer_id TEXT,
                file_size INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // Initialize the `chunks` table.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_chunk_hash TEXT,
            peer_id TEXT,
            is_cached BOOLEAN DEFAULT FALSE
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // Initialize the `nodes` table.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            peer_id TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // Initialize the `locked_contents` table with a unique constraint on (content_hash, chunk_index).
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS locked_contents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash TEXT,
                chunk_index INTEGER NOT NULL,
                chunk_size INTEGER NOT NULL,
                peer_id TEXT,
                UNIQUE(content_hash, chunk_index)
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ContentManager { pool })
    }

    /// Adds provided content to the `provided_contents` table.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    /// * `peer_id` - The ID of the peer providing the content.
    /// * `file_size` - The size of the file in bytes.
    /// * `file_name` - The name of the file.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error message.
    pub async fn add_provided_content(
        &self,
        content_hash: String,
        peer_id: String,
        file_size: u64,
        file_name: String,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO provided_contents (content_hash, peer_id, file_size, file_name)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(content_hash) DO UPDATE SET
                peer_id = excluded.peer_id,
                file_size = excluded.file_size,
                file_name = excluded.file_name
            "#,
        )
        .bind(content_hash)
        .bind(peer_id)
        .bind(file_size as i64)
        .bind(file_name)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn add_node(&self, peer_id: String) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO nodes (peer_id)
            VALUES (?)
            "#,
        )
        .bind(peer_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Locks a specific chunk of content, similar to pinning.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    /// * `chunk_index` - The index of the chunk.
    /// * `chunk_size` - The size of the chunk in bytes.
    /// * `peer_id` - The ID of the peer locking the chunk.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error message.
    pub async fn lock_content(
        &self,
        content_hash: String,
        chunk_index: i64,
        chunk_size: i64,
        peer_id: String,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO locked_contents (content_hash, chunk_index, chunk_size, peer_id)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(content_hash, chunk_index) DO UPDATE SET
                chunk_size = excluded.chunk_size,
                peer_id = excluded.peer_id
            "#,
        )
        .bind(content_hash)
        .bind(chunk_index)
        .bind(chunk_size)
        .bind(peer_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Unlocks content by removing the lock associated with a specific peer.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    /// * `peer_id` - The ID of the peer unlocking the content.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error message.
    pub async fn unlock_content(
        &self,
        content_hash: String,
        peer_id: String,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM locked_contents
            WHERE content_hash = ? AND peer_id = ?
            "#,
        )
        .bind(content_hash)
        .bind(peer_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Retrieves a list of `content_hash`es locked by a specific `peer_id`.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `content_hash`es or an error message.
    pub async fn get_peerid_locked_content(&self, peer_id: String) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            r#"
            SELECT content_hash FROM locked_contents
            WHERE peer_id = ?
            "#,
        )
        .bind(peer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let content_list = rows
            .into_iter()
            .filter_map(|row| row.get::<Option<String>, _>(0))
            .collect();

        Ok(content_list)
    }

    /// Retrieves a list of `content_hash`es provided by a specific `peer_id`.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `content_hash`es or an error message.
    pub async fn get_peerid_provided_content(
        &self,
        peer_id: String,
    ) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            r#"
            SELECT content_hash FROM provided_contents
            WHERE peer_id = ?
            "#,
        )
        .bind(peer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let content_list = rows
            .into_iter()
            .filter_map(|row| row.get::<Option<String>, _>(0))
            .collect();

        Ok(content_list)
    }

    /// Retrieves all provided content details given its hash.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tuples with content details or an error message.
    pub async fn get_provided_content(
        &self,
        content_hash: &String,
    ) -> Result<Vec<(i64, String, String, i64)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, content_hash, peer_id, file_size FROM provided_contents
            WHERE content_hash = ?
            "#,
        )
        .bind(content_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut content_list = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let content_hash: String = row.get("content_hash");
            let peer_id: String = row.get("peer_id");
            let file_size: i64 = row.get("file_size");
            content_list.push((id, content_hash, peer_id, file_size));
        }

        Ok(content_list)
    }

    /// Retrieves all nodes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `peer_id`es or an error message.
    pub async fn get_nodes(&self) -> Result<Vec<PeerId>, String> {
        let rows = sqlx::query(
            r#"
            SELECT peer_id FROM nodes
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut nodes = Vec::new();

        for row in rows {
            if let Some(peer_id_str) = row.get::<Option<String>, _>(0) {
                // Convert the string to PeerId
                match PeerId::from_str(&peer_id_str) {
                    Ok(peer_id) => nodes.push(peer_id),
                    Err(e) => return Err(format!("Failed to parse PeerId: {}", e)),
                }
            }
        }

        Ok(nodes)
    }

    /// Retrieves all locked content hashes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `content_hash`es or an error message.
    pub async fn get_locked_content(&self) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            r#"
            SELECT content_hash FROM locked_contents
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let content_list = rows
            .into_iter()
            .filter_map(|row| row.get::<Option<String>, _>(0))
            .collect();

        Ok(content_list)
    }

    /// Distributes file chunks among available peers in a round-robin fashion.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    /// * `peers` - A vector of available `PeerId`s.
    /// * `file_chunks` - A vector of file chunks as byte vectors.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tuples mapping `PeerId`s to `content_chunk_index`es or an error message.
    pub async fn distribute_file_chunks(
        &self,
        content_hash: String,
        peers: Vec<PeerId>,
        file_chunks: Vec<Vec<u8>>,
    ) -> Result<Vec<(PeerId, String)>, String> {
        if peers.is_empty() {
            return Err("No available peers to distribute chunks.".to_string());
        }

        let distributor_peers = peers;

        let mut peer_chunk_map = Vec::new();

        // Iterate over the chunks and assign them to peers
        for (i, chunk) in file_chunks.into_iter().enumerate() {
            let peer_index = (i as usize) % distributor_peers.len();
            let assigned_peer = distributor_peers[peer_index].clone();
            let content_chunk_index = format!("{}_chunk_{}", content_hash, i);

            // Add the peer and chunk info to the result
            peer_chunk_map.push((assigned_peer, content_chunk_index));
        }

        Ok(peer_chunk_map)
    }

    /// Caches a chunk at a specific peer by locking the content and writing the chunk to local storage.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    /// * `peer_id` - The ID of the peer.
    /// * `content_chunk_index` - The index identifier for the chunk.
    /// * `chunk_data` - The actual chunk data as a byte slice.
    /// * `chunk_index` - The index of the chunk.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error message.
    // async fn cache_chunk_at_peer(
    //     &self,
    //     content_hash: String,
    //     peer_id: String,
    //     content_chunk_index: String,
    //     chunk_data: &[u8],
    //     chunk_index: i64,
    // ) -> Result<(), String> {
    //     // Add the chunk metadata to the database (track which peer is responsible for this chunk)
    //     self.lock_content(content_hash.clone(), chunk_index, chunk_data.len() as i64, peer_id.clone())
    //         .await?;
    //     self.send_chunk_to_peer(
    //         &peer_id,
    //         &content_hash,
    //         &content_chunk_index,
    //         chunk_data,
    //         chunk_index
    //     ).await?;
    //
    //     // Cache the chunk locally at the peer
    //     cache_chunk_locally(&content_hash, &content_chunk_index, chunk_index, chunk_data).await?;
    //
    //     Ok(())
    // }

    /// Reads all cached chunks from local storage.
    ///
    ///
    /// # Returns
    ///
    /// A `Result` containing the chunk data as a byte vector or an error message.

    pub async fn read_cached_chunks(&self) -> Result<Vec<String>, String> {
        let cache_dir = create_cache_directory().await?;

        // Initialize a vector to store content hashes (filenames)
        let mut cached_contents = Vec::new();

        // Iterate over all files in the cache directory
        let mut entries = fs::read_dir(&cache_dir)
            .await
            .expect("Error reading directory");
        //let mut entries = tokio::fs::read_dir(&cache_dir).await.expect("Error reading directory");

        while let Ok(Some(entry)) = entries.next_entry().await {
            // let entry = entry.unwrap();
            let path = entry.path();

            // Only process files (ignore directories or invalid paths)
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(content_hash) = file_name.to_str() {
                        cached_contents.push(content_hash.to_string());
                    }
                }
            }
        }

        Ok(cached_contents)
    }

    /// Retrieves all chunks associated with a given file's `content_hash`.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tuples with chunk details or an error message.
    pub async fn get_chunks_for_file(
        &self,
        content_hash: String,
    ) -> Result<Vec<(i64, String, i64)>, String> {
        self.get_chunks_for_content(content_hash).await
    }

    /// Retrieves all chunks for a given `content_hash`.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - The hash of the content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tuples with chunk details or an error message.
    async fn get_chunks_for_content(
        &self,
        content_hash: String,
    ) -> Result<Vec<(i64, String, i64)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT chunk_index, peer_id, chunk_size FROM locked_contents
            WHERE content_hash = ?
            "#,
        )
        .bind(content_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut chunk_list = Vec::new();
        for row in rows {
            let chunk_index: i64 = row.get("chunk_index");
            let peer_id: String = row.get("peer_id");
            let chunk_size: i64 = row.get("chunk_size");
            chunk_list.push((chunk_index, peer_id, chunk_size));
        }

        Ok(chunk_list)
    }
    /// Splits a file into chunks of specified size asynchronously.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file to be split.
    /// * `chunk_size` - The size of each chunk in bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of byte vectors (chunks) or an error message.
    pub async fn split_file_into_chunks(
        &self,
        file_path: &str,
        chunk_size: usize,
    ) -> Result<Vec<Vec<u8>>, String> {
        let mut file = fs::File::open(file_path).await.map_err(|e| e.to_string())?;
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break; // End of file
            }

            let mut hasher = Sha256::new();
            hasher.update(&buffer[..bytes_read]);
            let result = hasher.finalize();
            let content_chunk_hash = hex::encode(result);
            sqlx::query("INSERT INTO chunks (content_chunk_hash) VALUES (?)")
                .bind(content_chunk_hash)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            chunks.push(buffer[..bytes_read].to_vec());
        }

        Ok(chunks)
    }
}

/// Caches a chunk locally at the specified location asynchronously.
///
/// # Arguments
///
/// * `content_hash` - The hash of the content.
/// * `content_chunk_index` - The index identifier for the chunk.
/// * `chunk_index` - The index of the chunk.
/// * `chunk_data` - The actual chunk data as a byte slice.
///
/// # Returns
///
/// A `Result` indicating success or containing an error message.
pub(crate) async fn cache_chunk_locally(
    content_chunk_index: &String,
    chunk_data: String,
) -> Result<(), String> {
    let cache_dir = create_cache_directory().await?;
    let chunk_file_path = cache_dir.join(content_chunk_index);
    let data = decode(chunk_data).expect("Error decoding chunk data");
    fs::write(&chunk_file_path, data)
        .await
        .map_err(|e| e.to_string())?;

    println!(
        "Cached chunk {} at {:?}",
        content_chunk_index, chunk_file_path,
    );

    Ok(())
}

/// Creates the cache directory if it doesn't exist and returns its path.
///
/// # Returns
///
/// A `Result` containing the `PathBuf` of the cache directory or an error message.
async fn create_cache_directory() -> Result<PathBuf, String> {
    let cache_dir = cache_dir().ok_or("Could not find cache directory")?;
    let boxpeer_cache_dir = cache_dir.join("boxpeer");

    if !boxpeer_cache_dir.exists() {
        fs::create_dir_all(&boxpeer_cache_dir)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(boxpeer_cache_dir)
}
