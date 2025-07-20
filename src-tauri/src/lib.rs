use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry, File};

use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

use audiotags::{Tag, Picture, MimeType};

mod core;

use crate::core::song::{ Song };
use crate::core::scan::{scan_dir, CError};
    

#[tauri::command]
fn read_directory(path: &str) -> Result<(), CError> {
    let dir = Path::new(path);
    if !dir.exists() { return Err(CError::InvalidPath)}

    let mut songs = Vec::new();
    let mut albums = Vec::new();

    scan_dir(&path, &mut songs, &mut albums);
    Ok(())
}















#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
