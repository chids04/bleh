use crate::{audio, db_dir, AppState};
use crate::core::song::{Album, Artist, ArtistType, Image, Song};

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, mpsc};
use std::collections::{HashMap, HashSet};

use tauri::State;
use uuid::Uuid;
use rusqlite::{Connection, Result, Transaction};

pub struct MusicLibrary {
    pub songs: HashMap<Uuid, Song>,
    pub albums: HashMap<Uuid, Album>,
    pub artist_manager: ArtistManager,
    pub player: audio::PlayerController,
    pub db_conn: Connection,
    pub required_covers: HashSet<Uuid>,
    pub folders: HashMap<i64, PathBuf>,
}

pub struct ArtistManager {
    pub artists: HashMap<Uuid, Artist>,
    pub known_artists: HashMap<Uuid, ArtistType>,

}


impl MusicLibrary {
    pub fn new() -> Self {
        //channels for communication between main thread and audio thread
        //rodio already uses a seperate thread but rodio structs arent send or sync so cant be stored in tauri app state
        //detached thread is used to mitigate this

        let dir = db_dir();
        let conn = match Connection::open(&dir) {
            Ok(c) => c,
            Err(e) => panic!("failed to init db {e}"),
        };
        
        if let Err(e) = init_db(&conn) {
            panic!("Failed to initialize database schema: {}", e);
        }

        println!("Loading music library from database...");

        let folders = match get_all_folders(&conn) {
            Ok(f) => {
                println!("loaded saved folders from databse {}", f.len());
                f
            }
            Err(e) => {
                println!("failed to load folders from databse {e}");
                HashMap::new()
            }
        };
        
        let songs = match get_all_songs(&conn) {
            Ok(s) => {
                println!("Loaded {} songs from database", s.len());
                s
            },
            Err(e) => {
                println!("Failed to load songs from database: {}", e);
                HashMap::new()
            }
        };
        
        let albums = match get_all_albums(&conn) {
            Ok(a) => {
                println!("Loaded {} albums from database", a.len());
                a
            },
            Err(e) => {
                println!("Failed to load albums from database: {}", e);
                HashMap::new()
            }
        };
        
        let artists = match get_all_artists(&conn) {
            Ok(a) => {
                println!("Loaded {} artists from database", a.len());
                a
            },
            Err(e) => {
                println!("Failed to load artists from database: {}", e);
                HashMap::new()
            }
        };
        
        let mut known_artists = HashMap::new();
        for (id, _) in &artists {
            known_artists.insert(*id, ArtistType::KnownArtist(*id));
        }
        println!("Initialized {} known artists", known_artists.len());

        let (sender, receiver) = mpsc::channel();
        let (sender2, receiver2) = mpsc::channel();

        thread::spawn(move || {
            audio::audio_thread_loop(receiver, sender2);
        });

        MusicLibrary {
            songs,
            albums,
            artist_manager: ArtistManager {
                artists,
                known_artists,
            },
            player: audio::PlayerController::new(sender, receiver2),
            db_conn: conn,
            required_covers: HashSet::new(),
            folders: folders,
            
        }
    }

    
    
}

pub fn init_db(conn: &Connection) -> Result<()>{
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    conn.execute(
    "CREATE TABLE IF NOT EXISTS artists (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL)",
        [],
    )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        )",
    [],
    )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS albums (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            cover_data BLOB
        )",
    []
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS album_artists (
            album_id TEXT NOT NULL,
            artist_id TEXT,                    
            artist_name TEXT NOT NULL,         
            is_primary BOOLEAN DEFAULT 0,     
            order_index INTEGER DEFAULT 0,    
            PRIMARY KEY(album_id, artist_name),
            FOREIGN KEY(album_id) REFERENCES albums(id) ON DELETE CASCADE,
            FOREIGN KEY(artist_id) REFERENCES artists(id) ON DELETE SET NULL
        )",
        []
    )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS songs (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            album_id TEXT NOT NULL,
            folder_id ID NOT NULL,
            cover_data BLOB,
            track_num INTEGER,
            disc_num INTEGER,
            path TEXT NOT NULL UNIQUE,
            duration REAL DEFAULT 0.0,

            FOREIGN KEY(artist_id) REFERENCES artists(id) ON DELETE CASCADE,
            FOREIGN KEY(album_id) REFERENCES albums(id) ON DELETE CASCADE
        )",
[]
    )?;

    conn.execute(
    "CREATE TABLE IF NOT EXISTS song_features (
            artist_id TEXT NOT NULL,
            song_id TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            PRIMARY KEY(song_id, artist_id),
            FOREIGN KEY(artist_id) REFERENCES artists(id),
            FOREIGN KEY(song_id) REFERENCES songs(id) ON DELETE CASCADE
        )",
[]
    )?;

    Ok(())
}

pub fn insert_folder_and_get_id<P: AsRef<Path>>(tx: &Transaction, path: P, state: State<AppState>) -> Result<i64, rusqlite::Error> {
    match tx.execute(
        "INSERT OR IGNORE INTO folders (path) VALUES (?1)",
        [path.as_ref().to_string_lossy()],
    )? {
        0 => {
            //folder already exists, get existing ID
            tx.query_row(
                "SELECT id FROM folders WHERE path = ?1",
                [path.as_ref().to_string_lossy()],
                |row| row.get(0)
            )
        }
        _ => {
            // new folder inserted
            let mut state = state.lock().unwrap();
            let id = tx.last_insert_rowid();
            state.folders.insert(id, path.as_ref().to_path_buf());
            Ok(id)
        }
    }
}


pub fn insert_song_to_db(
    tx: &Transaction,
    song: &Song,
    artists: &HashMap<Uuid, Artist>,
    albums: &HashMap<Uuid, Album>
) -> Result<(), rusqlite::Error> {

    if let Some(artist) = artists.get(&song.artist) {
        tx.execute(
            "INSERT OR IGNORE INTO artists (id, name) VALUES (?1, ?2)",
            (song.artist.to_string(), &artist.name),
        )?;
    }

    if let Some(album) = albums.get(&song.album) {
        tx.execute(
            "INSERT OR IGNORE INTO albums (id, name, cover_data) VALUES (?1, ?2, ?3)",
            (
                album.id.to_string(),
                &album.title,
                album.cover.as_ref().map(|img| &img.data),
            ),
        )?;

        insert_album_artists(tx, album.id, &album.artists)?;
    }

    tx.execute(
        "INSERT OR REPLACE INTO songs (id, title, artist_id, album_id, folder_id, track_num, disc_num, path, duration) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            song.id.to_string(),
            &song.title,
            song.artist.to_string(),
            song.album.to_string(),
            song.folder_id,
            song.track_num,
            song.disc_num,
            song.path.to_string_lossy(),
            song.duration,
        ),
    )?;

    if let Some(features) = &song.features {
        for (artist_id, artist_name) in features {
            tx.execute(
                "INSERT OR IGNORE INTO song_features (song_id, artist_id, artist_name) VALUES (?1, ?2, ?3)",
                (
                    song.id.to_string(),
                    artist_id.map(|id| id.to_string()),
                    artist_name,
                ),
            )?;
        }
    }

    Ok(())
}

pub fn insert_album_artists(
    conn: &Connection,
    album_id: Uuid,
    artists: &[(Option<Uuid>, String)]
) -> Result<(), rusqlite::Error> {
    for (index, (artist_id, artist_name)) in artists.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO album_artists (album_id, artist_id, artist_name, is_primary, order_index) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                album_id.to_string(),
                artist_id.map(|id| id.to_string()),
                artist_name,
                index == 0, 
                index as i32,
            ),
        )?;
    }
    Ok(())
}

pub fn get_all_folders(conn: &Connection) -> Result<HashMap<i64, PathBuf>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, path FROM folders")?;
    
    let artist_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;  
        let path: String = row.get(1)?;    
        let path = PathBuf::from(path);
            
        Ok((id, path))
    })?;

    
    let mut artists = HashMap::new();
    for artist_result in artist_iter {
        if let Ok(folder) = artist_result {
            artists.insert(folder.0, folder.1);
        }
        
    }
    Ok(artists)
}

pub fn get_all_artists(conn: &Connection) -> Result<HashMap<Uuid, Artist>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name FROM artists")?;
    
    let artist_iter = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;  
        let name: String = row.get(1)?;    
        

        let id = Uuid::parse_str(&id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?;
            
        Ok(Artist { id, name })
    })?;

    

    let mut artists = HashMap::new();
    for artist_result in artist_iter {
        if let Ok(artist) = artist_result {
            artists.insert(artist.id, artist);
        }
        
    }
    Ok(artists)
}


pub fn get_all_albums(conn: &Connection) -> Result<HashMap<Uuid, Album>, rusqlite::Error> {
    // get basic album info
    let mut stmt = conn.prepare("SELECT id, name, cover_data FROM albums")?;
    
    let album_iter = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let cover_data: Option<Vec<u8>> = row.get(2)?;
        
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?;
            
        Ok((id, name, cover_data))
    })?;

    let mut albums = HashMap::new();
    
    for album_result in album_iter {
        if let Ok((album_id, title, cover_data)) = album_result {
            let artists = get_album_artists(conn, album_id)?;
            
            let songs = get_album_songs(conn, album_id)?;
            
            let cover = cover_data.map(|data| Image {
                data,
                extension: "image/jpeg".to_string(),
            });
            
            let album = Album {
                id: album_id,
                title,
                artists,
                cover,
                songs,
            };
            
            albums.insert(album_id, album);
        }
    }

    Ok(albums)
}

pub fn get_album_artists(conn: &Connection, album_id: Uuid) -> Result<Vec<(Option<Uuid>, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT artist_id, artist_name FROM album_artists 
         WHERE album_id = ?1 
         ORDER BY order_index"
    )?;
    
    let artists: Vec<(Option<Uuid>, String)> = stmt.query_map([album_id.to_string()], |row| {
        let artist_id_str: Option<String> = row.get(0)?;
        let artist_name: String = row.get(1)?;
        
        let artist_id = artist_id_str
            .and_then(|s| Uuid::parse_str(&s).ok());
            
        Ok((artist_id, artist_name))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(artists)
}

pub fn get_album_songs(conn: &Connection, album_id: Uuid) -> Result<Vec<Uuid>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id FROM songs WHERE album_id = ?1")?;
    
    let songs: Vec<Uuid> = stmt.query_map([album_id.to_string()], |row| {
        let song_id_str: String = row.get(0)?;
        let song_id = Uuid::parse_str(&song_id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "song_id".to_string(), rusqlite::types::Type::Text))?;
        Ok(song_id)
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(songs)
}

pub fn get_all_songs(conn: &Connection) -> Result<HashMap<Uuid, Song>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.title, s.artist_id, s.album_id, s.folder_id, s.cover_data, 
                s.track_num, s.disc_num, s.path, s.duration
         FROM songs s")?;
    
    let songs_iter = stmt.query_map([], |row| {
        let id_str: String = row.get("id")?;
        let title: String = row.get("title")?;
        let artist_id_str: String = row.get("artist_id")?;
        let album_id_str: String = row.get("album_id")?;
        let folder_id: i64 = row.get("folder_id")?;
        let cover_data: Option<Vec<u8>> = row.get("cover_data")?;
        let track_num: u16 = row.get("track_num")?;
        let disc_num: u16 = row.get("disc_num")?;
        let path_str: String = row.get("path")?;
        let duration: f64 = row.get("duration").unwrap_or(0.0);
        
        // parse UUIDs
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?;
        let artist_id = Uuid::parse_str(&artist_id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "artist_id".to_string(), rusqlite::types::Type::Text))?;
        let album_id = Uuid::parse_str(&album_id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "album_id".to_string(), rusqlite::types::Type::Text))?;
        
        let cover = cover_data.map(|data| Image {
            data,
            extension: "image/jpg".to_string(),
        });
        
        let song = Song {
            id,
            title,
            artist: artist_id,
            album: album_id,
            features: None, 
            track_num,
            disc_num,
            cover,
            path: std::path::PathBuf::from(path_str),
            duration,
            folder_id,
        };
        
        Ok((id, song))
    })?;
    
    let mut songs = HashMap::new();
    for song_result in songs_iter {
        if let Ok((id, mut song)) = song_result {
            song.features = match get_song_features(conn, id) {
                Ok(features) => {
                    if features.is_empty() {
                        None
                    } else {
                        Some(features)
                    }
                },
                Err(e) => {
                    println!("Failed to load features for song {}: {}", id, e);
                    None
                }
            };
            
            songs.insert(id, song);
        }
    }
    
    Ok(songs)
}

pub fn get_song_features(conn: &Connection, song_id: Uuid) -> Result<Vec<(Option<Uuid>, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT artist_id, artist_name FROM song_features WHERE song_id = ?1"
    )?;
    
    let features: Vec<(Option<Uuid>, String)> = stmt.query_map([song_id.to_string()], |row| {
        let artist_id_str: String = row.get(0)?;
        let artist_name: String = row.get(1)?;
        
        let artist_id = Uuid::parse_str(&artist_id_str).ok();
        
        Ok((artist_id, artist_name))
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_connection_opens() {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory db");
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_init_db_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).expect("Failed to initialize db");

        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='artists'").unwrap();
        let mut rows = stmt.query(()).unwrap();
        assert!(rows.next().unwrap().is_some());

        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='albums'").unwrap();
        let mut rows = stmt.query(()).unwrap();
        assert!(rows.next().unwrap().is_some());

        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='songs'").unwrap();
        let mut rows = stmt.query(()).unwrap();
        assert!(rows.next().unwrap().is_some());

        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='song_features'").unwrap();
        let mut rows = stmt.query(()).unwrap();
        assert!(rows.next().unwrap().is_some());
    }
}
