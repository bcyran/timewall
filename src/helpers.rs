use anyhow::Result;
use std::{fs, io, path::Path};

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = fs::File::open(&path)?;
    let mut hasher = blake3::Hasher::new();
    io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();
    Ok(hash_bytes.to_hex().to_lowercase())
}
