use std::{fs, hash::Hasher, io::Read, path::Path};

use anyhow::Result;

use crate::wallpaper::Wallpaper;
use crate::{cache::Cache, wallpaper::unpack};

/// Unpacks HEIF files and loads them into `Wallpaper` structs, while transparently caching them.
#[derive(Debug)]
pub struct WallpaperLoader {
    cache: Cache,
}

impl WallpaperLoader {
    pub fn new() -> Self {
        Self {
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
            unpack(&path, &cache_dir).expect("wallpaper unpacking failed");
        }
        Wallpaper::load(&cache_dir).expect("malformed wallpaper cache")
    }

    /// Clear the wallpaper loader cache. Optionally skips one wallpaper with a given pahth.
    pub fn clear_cache<P: AsRef<Path>>(&mut self, skip_wall_path: Option<P>) {
        let mut entries_to_clear = self.cache.entries.clone();

        if let Some(skip_wall_path) = skip_wall_path {
            let skip_wall_hash = hash_file(&skip_wall_path).expect("wallpaper hashing failed");
            entries_to_clear.remove(&skip_wall_hash);
        }

        for hash in entries_to_clear {
            self.cache.remove_entry(&hash);
        }
    }
}

fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    const BUFFER_LEN: usize = 1024;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut file = fs::File::open(&path)?;
    let mut hasher = seahash::SeaHasher::new();

    loop {
        let read_count = file.read(&mut buffer)?;
        hasher.write(&buffer[..read_count]);

        if read_count != BUFFER_LEN {
            break;
        }
    }

    let hash_bytes = hasher.finish();
    Ok(format!("{hash_bytes:x}"))
}
