use std::{fs, io, path::Path};

use anyhow::Result;

use crate::wallpaper::Wallpaper;
use crate::{cache::Cache, wallpaper::unpack_heic};

/// Unpacks HEIC files and loads them into `Wallpaper` structs, while transparently caching them.
#[derive(Debug)]
pub struct WallpaperLoader {
    cache: Cache,
}

impl WallpaperLoader {
    pub fn new() -> Self {
        WallpaperLoader {
            cache: Cache::find("wallpapers"),
        }
    }

    /// Load given file into `Wallpaper` struct.
    ///
    /// Each loaded file is persistently cached and will be loaded from cache if requested again.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Wallpaper {
        let hash = hash_file(&path).expect("wallpaper hashing failed");
        let cache_dir = self.cache.entry(&hash);
        if cache_dir.read_dir().unwrap().next().is_none() {
            unpack_heic(&path, &cache_dir).expect("wallpaper unpacking failed");
        }
        Wallpaper::load(&cache_dir).expect("malformed wallpaper cache")
    }
}

fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = fs::File::open(&path)?;
    let mut hasher = blake3::Hasher::new();
    io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();
    Ok(hash_bytes.to_hex().to_lowercase())
}
