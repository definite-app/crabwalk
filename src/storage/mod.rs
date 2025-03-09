use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for S3 storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// S3 access key
    pub access_key: Option<String>,
    /// S3 secret key
    pub secret_key: Option<String>,
    /// S3 endpoint URL
    pub endpoint_url: Option<String>,
    /// S3 region name
    pub region_name: Option<String>,
    /// Folder name for database backup
    pub db_folder_name: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            access_key: None,
            secret_key: None,
            endpoint_url: None,
            region_name: None,
            db_folder_name: "db".to_string(),
        }
    }
}

/// Backup the DuckDB database to S3
///
/// # Arguments
///
/// * `database_path` - Path to the DuckDB database file
/// * `s3_config` - S3 configuration
///
/// # Returns
///
/// * `Result<()>` - Success or error
#[cfg(feature = "s3")]
pub fn backup(database_path: &str, s3_config: &S3Config) -> Result<()> {
    use duckdb::Connection;
    use rusoto_core::Region;
    use rusoto_s3::{PutObjectRequest, S3Client, S3};
    use std::io::Read;
    
    tracing::info!(
        "Backing up the DuckDB database to {}/{}",
        s3_config.bucket,
        s3_config.db_folder_name
    );
    
    // Create temporary directory to export the database
    let temp_dir = TempDir::new()
        .context("Failed to create temporary directory")?;
    let local_db_path = temp_dir.path().join(&s3_config.db_folder_name);
    fs::create_dir_all(&local_db_path)
        .context(format!("Failed to create directory: {}", local_db_path.display()))?;
    
    // Export the database to the temporary directory
    let conn = Connection::open(database_path)
        .context(format!("Failed to connect to DuckDB database: {}", database_path))?;
    conn.execute(
        &format!("EXPORT DATABASE '{}' (FORMAT 'parquet')", local_db_path.display()),
        [],
    )
    .context("Failed to export database")?;
    
    // Configure S3 client
    let region = if let Some(endpoint) = &s3_config.endpoint_url {
        Region::Custom {
            name: s3_config.region_name.clone().unwrap_or_else(|| "custom".to_string()),
            endpoint: endpoint.clone(),
        }
    } else {
        s3_config
            .region_name
            .as_ref()
            .map(|name| name.parse::<Region>().unwrap_or(Region::UsEast1))
            .unwrap_or(Region::UsEast1)
    };
    
    let s3_client = if let (Some(access_key), Some(secret_key)) = (&s3_config.access_key, &s3_config.secret_key) {
        let credentials_provider = rusoto_core::credential::StaticProvider::new_minimal(
            access_key.clone(),
            secret_key.clone(),
        );
        S3Client::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            credentials_provider,
            region,
        )
    } else {
        S3Client::new(region)
    };
    
    // Upload files to S3
    for entry in walkdir::WalkDir::new(&local_db_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path
                .strip_prefix(&temp_dir)
                .context("Failed to get relative path")?;
            let key = relative_path.to_string_lossy().to_string();
            
            let mut file = fs::File::open(path)
                .context(format!("Failed to open file: {}", path.display()))?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .context(format!("Failed to read file: {}", path.display()))?;
            
            let put_request = PutObjectRequest {
                bucket: s3_config.bucket.clone(),
                key,
                body: Some(contents.into()),
                ..Default::default()
            };
            
            s3_client
                .put_object(put_request)
                .sync()
                .context("Failed to upload file to S3")?;
        }
    }
    
    tracing::info!("Backup completed successfully");
    
    Ok(())
}

/// Backup function placeholder for when S3 feature is disabled
#[cfg(not(feature = "s3"))]
pub fn backup(_database_path: &str, _s3_config: &S3Config) -> Result<()> {
    tracing::warn!("S3 support is not enabled. Build with --features s3 to enable.");
    Ok(())
}

/// Restore the DuckDB database from S3
///
/// # Arguments
///
/// * `database_path` - Path to the DuckDB database file
/// * `s3_config` - S3 configuration
/// * `overwrite` - Whether to overwrite existing database
///
/// # Returns
///
/// * `Result<()>` - Success or error
#[cfg(feature = "s3")]
pub fn restore(database_path: &str, s3_config: &S3Config, overwrite: bool) -> Result<()> {
    use duckdb::Connection;
    use rusoto_core::Region;
    use rusoto_s3::{GetObjectRequest, ListObjectsV2Request, S3Client, S3};
    use std::io::Read;
    
    tracing::info!(
        "Restoring the DuckDB database from {}/{}",
        s3_config.bucket,
        s3_config.db_folder_name
    );
    
    // Check if database exists and should be overwritten
    let db_path = Path::new(database_path);
    if db_path.exists() {
        if overwrite {
            fs::remove_file(db_path)
                .context(format!("Failed to remove existing database: {}", database_path))?;
        } else {
            return Err(anyhow::anyhow!(
                "Database file already exists. Use --overwrite to replace it."
            ));
        }
    }
    
    // Create temporary directory to download the database
    let temp_dir = TempDir::new()
        .context("Failed to create temporary directory")?;
    let local_db_path = temp_dir.path().join(&s3_config.db_folder_name);
    fs::create_dir_all(&local_db_path)
        .context(format!("Failed to create directory: {}", local_db_path.display()))?;
    
    // Configure S3 client
    let region = if let Some(endpoint) = &s3_config.endpoint_url {
        Region::Custom {
            name: s3_config.region_name.clone().unwrap_or_else(|| "custom".to_string()),
            endpoint: endpoint.clone(),
        }
    } else {
        s3_config
            .region_name
            .as_ref()
            .map(|name| name.parse::<Region>().unwrap_or(Region::UsEast1))
            .unwrap_or(Region::UsEast1)
    };
    
    let s3_client = if let (Some(access_key), Some(secret_key)) = (&s3_config.access_key, &s3_config.secret_key) {
        let credentials_provider = rusoto_core::credential::StaticProvider::new_minimal(
            access_key.clone(),
            secret_key.clone(),
        );
        S3Client::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            credentials_provider,
            region,
        )
    } else {
        S3Client::new(region)
    };
    
    // List objects in the S3 bucket
    let prefix = format!("{}/", s3_config.db_folder_name);
    let list_request = ListObjectsV2Request {
        bucket: s3_config.bucket.clone(),
        prefix: Some(prefix),
        ..Default::default()
    };
    
    let list_result = s3_client
        .list_objects_v2(list_request)
        .sync()
        .context("Failed to list objects in S3 bucket")?;
    
    if let Some(objects) = list_result.contents {
        for object in objects {
            if let Some(key) = object.key {
                // Get object from S3
                let get_request = GetObjectRequest {
                    bucket: s3_config.bucket.clone(),
                    key: key.clone(),
                    ..Default::default()
                };
                
                let get_result = s3_client
                    .get_object(get_request)
                    .sync()
                    .context(format!("Failed to get object from S3: {}", key))?;
                
                // Save object to local file
                let relative_path = key.strip_prefix(&s3_config.db_folder_name)
                    .unwrap_or(&key);
                let local_path = local_db_path.join(relative_path);
                
                // Ensure directory exists
                if let Some(parent) = local_path.parent() {
                    fs::create_dir_all(parent)
                        .context(format!("Failed to create directory: {}", parent.display()))?;
                }
                
                // Write file
                if let Some(mut body) = get_result.body {
                    let mut content = Vec::new();
                    body.read_to_end(&mut content)
                        .context(format!("Failed to read S3 object body: {}", key))?;
                    
                    let mut file = fs::File::create(&local_path)
                        .context(format!("Failed to create file: {}", local_path.display()))?;
                    file.write_all(&content)
                        .context(format!("Failed to write to file: {}", local_path.display()))?;
                }
            }
        }
    }
    
    // Import the database
    let conn = Connection::open(database_path)
        .context(format!("Failed to connect to DuckDB database: {}", database_path))?;
    conn.execute(
        &format!("IMPORT DATABASE '{}'", local_db_path.display()),
        [],
    )
    .context("Failed to import database")?;
    
    tracing::info!("Restore completed successfully");
    
    Ok(())
}

/// Restore function placeholder for when S3 feature is disabled
#[cfg(not(feature = "s3"))]
pub fn restore(_database_path: &str, _s3_config: &S3Config, _overwrite: bool) -> Result<()> {
    tracing::warn!("S3 support is not enabled. Build with --features s3 to enable.");
    Ok(())
}