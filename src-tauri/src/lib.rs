pub mod core;
pub mod state;

use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use uuid::Uuid;

use crate::core::scan::{scan_dir, remove_folder};
use crate::core::song::{Album, Song, SongDto, Artist, Image};
use crate::core::audio;
use crate::state::MusicLibrary;

use serde::{Deserialize, Serialize};
use tauri::{Manager, State, AppHandle, Emitter};

pub type AppState = Mutex<MusicLibrary>;

pub fn db_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("db");

    if let Err(e) = fs::create_dir_all(&path) {
        panic!("failed to create db directory: {e}");
    }

    path.push("library.db"); 

    path
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SongEvent {
    PlayingSong {
        title: String,
        artist: (Uuid, String),
        features: Option<Vec<(Option<Uuid>, String)>>,
        album: (Uuid, String),
        duration: f64,
    }
}



#[derive(Serialize, Deserialize)]
pub struct SongToSend {
    pub id: Uuid,
    pub title: String,
    pub artist: (Uuid, String),
    pub album: (Uuid, String),
    pub features: Option<Vec<(Option<Uuid>, String)>>,
    pub track_num: u16,
    pub disc_num: u16,
    pub cover: Option<Image>,
    pub path: PathBuf,
    pub duration: f64
}

#[tauri::command]
fn seek_to(state: State<AppState>, pos: f64) {
    let state = state.lock().unwrap();
    state.player.seek(pos);
}
#[tauri::command]
fn toggle_play(state: State<AppState>) {
    let state = state.lock().unwrap();
    state.player.toggle_play();
}

#[tauri::command]
fn play_song(app: AppHandle, id: &str) -> Result<(), String>{
    let state = app.state::<AppState>();

    let state = state.lock().unwrap();

    let uuid = match Uuid::parse_str(id) {
        Ok(u) => u,
        Err(e) => return Err("invalid song id".into()),
    };

    if let Some(s) = state.songs.get(&uuid) {
        state.player.play_now(s.clone());

        let artist_string = match state.artist_manager.artists.get(&s.artist) {
            Some(a) => &a.name,
            None => "Unknown Artist"
            
        };

        let (album_string, cover): (String, Option<Image>) = match state.albums.get(&s.album) {
            Some(a) => {
                if let Some(c) = &a.cover {
                    (a.title.clone(), Some(c.clone()))
                }
                else{
                    (a.title.clone(), None)
                }
            }

            None => ("Unknown Artist".into(), None)
            
        };


        let msg = SongEvent::PlayingSong {
            title: s.title.clone(),
            artist: (s.artist.clone(), artist_string.into()),
            album: (s.album.clone(), album_string.into()),
            features: s.features.clone(),
            duration: s.duration,
        };

        println!("{:?}", &msg);


        app.emit("playing-song", &msg).unwrap();
    }
    
    // if let Some(s) = state.songs.get(index) {
    //     state.player.play_now(s.clone());
        
    //     let msg = SongEvent::PlayingSong { 
    //         title: s.title.clone(), 
    //         artists: {
    //             let mut a = s.artist.clone();
    //             if let Some(f) = &s.features{
    //                 let f_str = f.join(",");
    //                 a.push_str(", ");
    //                 a.push_str(&f_str);
    //                 a
    //             }
    //             else{
    //                 a
    //             }
    //         },
    //         cover: s.album.cover.clone()
    //     };
    //     app.emit("playing-song", &msg).unwrap();
    // }
    // else{
    //     return Err("requested song does not exist".into());
    // }

    Ok(())

}

#[tauri::command]
fn get_songs(state: State<AppState>) -> Vec<SongToSend>{
    let mut state = state.lock().unwrap();

    //for now i load all the songs into memory
    //this isnt scalable so i wil lhave to think up some buffered appraoch probs
    let mut required_album_uuids = Vec::new();

    let songs: Vec<SongToSend> = state.songs.values().map(|song| {

        let artist_string = match state.artist_manager.artists.get(&song.artist) {
            Some(a) => &a.name,
            None => "Unknown Artist"
            
        };

        let album_string = match state.albums.get(&song.album) {
            Some(a) => &a.title,
            None => "Unknown Artist"
            
        };

        required_album_uuids.push(song.album);

        SongToSend {
            id: song.id.clone(),
            title: song.title.clone(),
            artist: (song.artist, artist_string.to_string()),
           album: (song.album, album_string.to_string()),
            features: song.features.clone(),
            track_num: song.track_num,
            disc_num: song.disc_num,
            cover: song.cover.clone(),
            path: song.path.clone(),
            duration: song.duration,
        }
    }).collect();

    for album_uuid in required_album_uuids {
        state.required_covers.insert(album_uuid);
    }

    songs
}

#[tauri::command]
fn get_artists(state: State<AppState>) -> Vec<Artist> {
    let state = state.lock().unwrap();

    let artist_vec = state.artist_manager.artists
        .iter()
        .map(|a| a.1.clone())
        .collect();

    artist_vec
}


#[tauri::command]
fn get_albums(state: State<AppState>) -> Vec<Album> {
    let state = state.lock().unwrap();

    let album_vec = state.albums
        .iter()
        .map(|a| a.1.clone())
        .collect();

    album_vec
}

#[tauri::command]
fn get_covers(state: State<AppState>) -> HashMap<Uuid, Image> {
    let state = state.lock().unwrap();

    let mut covers = HashMap::new();

    for uuid in &state.required_covers {
        if let Some(a) = state.albums.get(uuid){
            if let Some(c) = &a.cover {
                covers.insert(*uuid, c.clone());
            }
        }
    }

    covers
}

#[tauri::command] 
fn get_cover(state: State<AppState>, id: &str) -> Option<Image> {
    let state = state.lock().unwrap();

    let uuid = match Uuid::parse_str(id) {
        Ok(u) => u,
        Err(e) => {
            println!("invalid uuid: {e}");
            return None;
        },
    };

    if let Some(a) = state.albums.get(&uuid) {
        if let Some(c) = &a.cover {
            return Some(c.clone())
        }
    }

    None
}

#[tauri::command]
fn read_directory(state: State<AppState>, path: &str) -> Result<(), String> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(String::from("invalid path"));
    }

    scan_dir(&path, state.clone());

    let state = state.lock().unwrap();
    state.songs.iter().for_each(|s| println!("{}", s.1));

    Ok(())
}

#[tauri::command]
fn delete_directory(state: State<AppState>, id: i64) {
    remove_folder(state, id);
}

#[tauri::command]
fn get_directories(state: State<AppState>) -> HashMap<i64, PathBuf> {
    let state = state.lock().unwrap();
    state.folders.clone()
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.manage(Mutex::new(MusicLibrary::new()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![read_directory, get_songs, get_covers, get_artists, get_albums, get_cover, play_song, toggle_play, delete_directory, get_directories, seek_to])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
