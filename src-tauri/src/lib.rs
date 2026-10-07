#[cfg(target_os = "macos")]
mod background;
#[cfg(target_os = "macos")]
mod commands;
pub mod db;
pub mod delete;
#[cfg(target_os = "macos")]
mod desktop;
pub mod error;
#[cfg(target_os = "macos")]
mod events;
pub mod models;
#[cfg(target_os = "macos")]
mod permissions;
pub mod service;
#[cfg(target_os = "macos")]
mod settings;
pub mod state;
#[cfg(target_os = "macos")]
mod tray;
#[cfg(target_os = "macos")]
pub use desktop::run;

#[cfg(test)]
pub(crate) fn test_directory() -> tempfile::TempDir {
    let directory = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    tempfile::tempdir_in(directory).unwrap()
}
