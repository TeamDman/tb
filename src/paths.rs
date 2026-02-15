use directories_next::ProjectDirs;
use eyre::bail;
use std::env;
use std::path::{Path, PathBuf};

pub const APP_HOME_ENV_VAR: &str = "TB_HOME_DIR";
pub const APP_CACHE_ENV_VAR: &str = "TB_CACHE_DIR";

#[derive(Clone, Debug)]
pub struct AppHome(PathBuf);

impl AppHome {
    pub fn resolve() -> eyre::Result<Self> {
        if let Ok(path) = env::var(APP_HOME_ENV_VAR) {
            return Ok(Self(PathBuf::from(path)));
        }
        if let Some(dirs) = ProjectDirs::from("", "teamdman", "tb") {
            return Ok(Self(dirs.config_dir().to_path_buf()));
        }
        bail!("Could not determine app home directory")
    }

    pub fn ensure_dir(&self) -> eyre::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct CacheHome(PathBuf);

impl CacheHome {
    pub fn resolve() -> eyre::Result<Self> {
        if let Ok(path) = env::var(APP_CACHE_ENV_VAR) {
            return Ok(Self(PathBuf::from(path)));
        }
        if let Some(dirs) = ProjectDirs::from("", "teamdman", "tb") {
            return Ok(Self(dirs.cache_dir().to_path_buf()));
        }
        bail!("Could not determine app cache directory")
    }

    pub fn ensure_dir(&self) -> eyre::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

pub fn app_home() -> eyre::Result<AppHome> {
    AppHome::resolve()
}

pub fn cache_home() -> eyre::Result<CacheHome> {
    CacheHome::resolve()
}
