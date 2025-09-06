use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::{Arc, RwLock},
};

use rusqlite::{params, Connection, Transaction};
use tauri::State;
use audiotags::{Picture, Tag};
use metadata::media_file::MediaFileMetadata;
use uuid::Uuid;

use crate::db_dir;
use crate::state::{get_all_albums, get_all_artists, insert_folder_and_get_id};
use crate::AppState;
use crate::state::{init_db, insert_song_to_db};
use crate::core::song::{Album, Artist, ArtistType, Image, Song};


#[derive(Debug)]
pub enum CError {
    InvalidTag(PathBuf),
    InvalidPath,
}

fn find_or_create_album(
    albums: &mut HashMap<Uuid, Album>,
    title: &str,
    parsed_artists: &[String],
    cover: Option<Image>,
    id: &Uuid,
    known_uuid: &Uuid,
    artists: &HashMap<Uuid, Artist>

) -> Uuid {

    for album in albums.values() {
        if album.title == title
            && album.artists.len() == parsed_artists.len()
            && album
                .artists
                .iter()
                .zip(parsed_artists.iter())
                .all(|(a, b)| a.1 == *b)
        {
            return album.id;
        }
    }

    //for the album artists, for each artist i create two new artists,
    //for now, i just add the known uuid of the first artist whilst i figure out a strat to solve duplicates
    let mut album_artists = Vec::new();
    if let Some(a) = artists.get(known_uuid) {
        album_artists.push((Some(*known_uuid), a.name.clone()));
    }

    //add remaining artists from the slice, but skip duplicates
    for artist_name in parsed_artists {
        let already_exists = album_artists.iter().any(|(_, name)| name == artist_name);
        if !already_exists {
            album_artists.push((None, artist_name.clone()));
        }
    }
    

    let album = Album {
        id: Uuid::new_v4(),
        title: title.to_string(),
        artists: album_artists,
        songs: vec![*id],
        cover,
    };

    let id = album.id;
    albums.insert(album.id, album);
    id
}

//for artists, we check to see if we already have an artist with the same name for now
//in future, we may need to consider artists that have the same name (unlikely but happens, e.g there are two artists called Russ)
fn find_or_create_artist(artists: &mut HashMap<Uuid, Artist>, known_artists: &mut HashMap<Uuid, ArtistType>, name: &str) -> Uuid {
    for artist in artists.values() {
        if artist.name == name {
            return artist.id;
        }
    }

    let id = Uuid::new_v4();

    let new_artist = Artist {
        id,
        name: name.into()
    };

    
    artists.insert(id, new_artist);
    known_artists.insert(id, ArtistType::KnownArtist(id));

    id
}

pub fn parse_file<P: AsRef<Path>>(
    path: P,
    albums: &mut HashMap<Uuid, Album>,
    artists: &mut HashMap<Uuid, Artist>,
    known_artists: &mut HashMap<Uuid, ArtistType>,
    folder_id: i64
) -> Result<Song, CError> {
    let tag = match Tag::new().read_from_path(&path) {
        Ok(t) => t,
        Err(e) => {
            println!(
                "parse_file() tag error for {}: {e}",
                path.as_ref().display()
            );
            return Err(CError::InvalidTag(path.as_ref().to_path_buf()));
        }
    };


    let title = tag.title().unwrap_or("unknown song");
    let artist = tag.artist().unwrap_or(title);

    let artist_uuid = find_or_create_artist(artists, known_artists,artist);

    let album_title = tag.album_title().unwrap_or("unknown album");

    let album_artists = match tag.album_artists() {
        Some(a) => a.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
        None => vec![artist.to_string()],
    };

    let cover: Option<Image> = tag.album_cover().map(|img| img.into());

    let song_id = Uuid::new_v4();

    let album = find_or_create_album(albums, album_title, &album_artists, cover, &song_id, &artist_uuid, &artists);

    let features = if let Some(mut artists_list) = tag.artists() {
        artists_list.retain(|a| *a !=artist);

        if artists_list.is_empty() {
            None
        } else {
            Some(artists_list.iter().map(|a| (None, a.to_string())).collect::<Vec<(Option<Uuid>, String)>>())
        }
    } else {
        None
    };

    let track_num = tag.track_number().unwrap_or(1);
    let disc_num = tag.disc_number().unwrap_or(1);

    let duration = match MediaFileMetadata::new(&path) {
        Ok(m) => {
            if let Some(d) = m.duration {
                if let Ok(d_float) = parse_duration_to_seconds(&d){
                    d_float
                }
                else{
                    0.0
                }
            }
            else{
                0.0
            }
        },

        Err(e) => {
            println!("failed to get duration of {title} by {artist}, defaulting to 0: {e}");
            0.0
        }
    };

    let song = Song {
        id: Uuid::new_v4(),
        title: title.to_string(),
        artist: artist_uuid,
        album,
        features,
        track_num,
        disc_num,
        cover: None,
        path: path.as_ref().to_path_buf(),
        duration,
        folder_id,
    };

    Ok(song)
}




pub fn scan_dir<P: AsRef<Path>>(dir: P, state: State<AppState>) {
    let mut songs = HashMap::new();
    let mut known_artists: HashMap<Uuid, ArtistType> = HashMap::new();

    //start sqlite transaction here
    let db_path = db_dir();
    
    let mut scann_conn = match Connection::open(&db_path) {
        Ok(conn) => {
            if let Err(e) = init_db(&conn) {
                println!("failed to validate/create database schema {e}");
                return;
            }
            else{
                conn
            }
        }
        Err(e) => {
            println!("failed to open sqlite db connection for scanning: {e}");
            return;
        }
    };
    let mut albums = match get_all_albums(&scann_conn){
        Ok(a) => a,
        Err(e) => {
            println!("failed to read albums from db, continuing with limited information {e}");
            HashMap::new()
        }
    };

    //here i need to get the artists
    let mut artists = match get_all_artists(&scann_conn) {
        Ok(a) => a,
        Err(e) => {
            println!("failed to get all artists from db, continuing with limited information {e}");
            HashMap::new()
        }
    };


    let mut dir_queue = VecDeque::new();
    dir_queue.push_back(dir.as_ref().to_path_buf());

    //start a transaction for batch inserts
    let tx = match scann_conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            println!("failed to start sqlite db transaction: {}", e);
            return;
        }
    };

    let folder_id = match insert_folder_and_get_id(&tx, &dir, state.clone()){
        Ok(f) => f,
        Err(e) => {
            println!("failed to load folder id from sqlite db {e}");
            return;
        }
    };

    while let Some(current_dir) = dir_queue.pop_front() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(err) => {
                println!(
                    "failed to read directory {}: {}",
                    current_dir.display(),
                    err
                );
                continue;
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    println!("failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if path.is_file() {
                match parse_file(&path, &mut albums, &mut artists, &mut known_artists, folder_id) {
                    Ok(song) => {
                        if let Err(e) = insert_song_to_db(&tx, &song, &artists, &albums) {
                            println!("Failed to insert song to DB: {}", e);
                        }
                        songs.insert(song.id, song);
                    }
                    Err(e) => {
                        println!("metadata extraction failed for {}: {:?}", path.display(), e);
                    }
                }
            } else if path.is_dir() {
                dir_queue.push_back(path);
            } else {
                println!("Unsupported file type: {}", path.display());
            }
        }
    }


    // commit the transaction
    if let Err(e) = tx.commit() {
        println!("Failed to commit scan transaction: {}", e);
        return;
    }

    // update in-memory state
    let mut state = state.lock().unwrap();
    state.songs = songs;
    state.albums = albums;
    state.artist_manager.artists = artists;
    state.artist_manager.known_artists = known_artists;

    
}


pub fn remove_folder(state: State<AppState>, id: i64) {
    let dir = db_dir();
    let mut conn = match Connection::open(&dir) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to open sqlite db: {}", e);
            return;
        }
    };

    if let Err(e) = init_db(&conn) {
        println!("Failed to initialize database schema: {}", e);
        return;
    }

    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            println!("Failed to start transaction: {}", e);
            return;
        }
    };

    let mut songs_to_delete = Vec::new();
    let mut albums_to_check = HashSet::new();
    let mut artists_to_check = HashSet::new();

    // Get all affected data in one pass
    {
        let mut stmt = match tx.prepare("SELECT id, album_id, artist_id FROM songs WHERE folder_id = ?1") {
            Ok(stmt) => stmt,
            Err(e) => {
                println!("Failed to prepare statement: {}", e);
                return;
            }
        };
        
        let rows = match stmt.query_map([id], |row| {
            let song_id: String = row.get(0)?;
            let album_id: String = row.get(1)?;
            let artist_id: String = row.get(2)?;
            
            Ok((
                Uuid::parse_str(&song_id).map_err(|_| rusqlite::Error::InvalidColumnType(0, "song_id".to_string(), rusqlite::types::Type::Text))?,
                Uuid::parse_str(&album_id).map_err(|_| rusqlite::Error::InvalidColumnType(1, "album_id".to_string(), rusqlite::types::Type::Text))?,
                Uuid::parse_str(&artist_id).map_err(|_| rusqlite::Error::InvalidColumnType(2, "artist_id".to_string(), rusqlite::types::Type::Text))?,
            ))
        }) {
            Ok(rows) => rows,
            Err(e) => {
                println!("Failed to execute query: {}", e);
                return;
            }
        };

        for row in rows {
            match row {
                Ok((song_id, album_id, artist_id)) => {
                    songs_to_delete.push(song_id);
                    albums_to_check.insert(album_id);
                    artists_to_check.insert(artist_id);
                }
                Err(e) => {
                    println!("Error processing row: {}", e);
                    return;
                }
            }
        }
    }

    //get orphaned albums/artists before deletion
    let mut albums_to_delete = Vec::new();
    let mut artists_to_delete = Vec::new();

    //check which albums will become orphaned
    for album_id in &albums_to_check {
        let count: i32 = match tx.query_row(
            "SELECT COUNT(*) FROM songs WHERE album_id = ?1 AND folder_id != ?2",
            [album_id.to_string(), id.to_string()],
            |row| row.get(0)
        ) {
            Ok(count) => count,
            Err(e) => {
                println!("Failed to check album orphan status: {}", e);
                return;
            }
        };
        
        if count == 0 {
            albums_to_delete.push(*album_id);
        }
    }

    //check which artists will become orphaned
    for artist_id in &artists_to_check {
        let song_count: i32 = match tx.query_row(
            "SELECT COUNT(*) FROM songs WHERE artist_id = ?1 AND folder_id != ?2",
            [artist_id.to_string(), id.to_string()],
            |row| row.get(0)
        ) {
            Ok(count) => count,
            Err(e) => {
                println!("Failed to check artist song count: {}", e);
                return;
            }
        };
        
        //check if artist has album associations with albums that wont be deleted
        let album_count: i32 = match tx.query_row(
            "SELECT COUNT(*) FROM album_artists aa 
             WHERE aa.artist_id = ?1 
             AND aa.album_id NOT IN (
                 SELECT DISTINCT album_id FROM songs WHERE folder_id = ?2
             )",
            [artist_id.to_string(), id.to_string()],
            |row| row.get(0)
        ) {
            Ok(count) => count,
            Err(e) => {
                println!("Failed to check artist album count: {}", e);
                return;
            }
        };
        
        let feature_count: i32 = match tx.query_row(
            "SELECT COUNT(*) FROM song_features sf 
             JOIN songs s ON sf.song_id = s.id 
             WHERE sf.artist_id = ?1 AND s.folder_id != ?2",
            [artist_id.to_string(), id.to_string()],
            |row| row.get(0)
        ) {
            Ok(count) => count,
            Err(e) => {
                println!("Failed to check artist feature count: {}", e);
                return;
            }
        };

        println!("Artist {}: songs={}, albums={}, features={}", 
                 artist_id, song_count, album_count, feature_count);

        if song_count == 0 && album_count == 0 && feature_count == 0 {
            artists_to_delete.push(*artist_id);
            println!("Marking artist {} for deletion", artist_id);
        }
    }

    //do all the deletions
    if let Err(e) = tx.execute("DELETE FROM song_features WHERE song_id IN (SELECT id FROM songs WHERE folder_id = ?1)", [id]) {
        println!("Failed to delete song features: {}", e);
        return;
    }

    if let Err(e) = tx.execute("DELETE FROM songs WHERE folder_id = ?1", [id]) {
        println!("Failed to delete songs: {}", e);
        return;
    }
    
    for album_id in &albums_to_delete {
        if let Err(e) = tx.execute("DELETE FROM albums WHERE id = ?1", [album_id.to_string()]) {
            println!("Failed to delete album {}: {}", album_id, e);
            return;
        }
        if let Err(e) = tx.execute("DELETE FROM album_artists WHERE album_id = ?1", [album_id.to_string()]) {
            println!("Failed to delete album artists for {}: {}", album_id, e);
            return;
        }
    }
    
    for artist_id in &artists_to_delete {
        if let Err(e) = tx.execute("DELETE FROM artists WHERE id = ?1", [artist_id.to_string()]) {
            println!("Failed to delete artist {}: {}", artist_id, e);
            return;
        }
    }
    
    if let Err(e) = tx.execute("DELETE FROM folders WHERE id = ?1", [id]) {
        println!("Failed to delete folder: {}", e);
        return;
    }
    
    //commit transaction
    if let Err(e) = tx.commit() {
        println!("Failed to commit transaction: {}", e);
        return;
    }

    //update in-memory state
    {
        let mut state = state.lock().unwrap();
        
        for song_id in &songs_to_delete {
            state.songs.remove(song_id);
        }
        
        for album_id in &albums_to_delete {
            state.albums.remove(album_id);
        }
        
        for artist_id in &artists_to_delete {
            state.artist_manager.artists.remove(artist_id);
            state.artist_manager.known_artists.remove(artist_id);
        }
        
        state.folders.remove(&id);
        
        println!("updated in-memory state: removed {} songs, {} albums, {} artists", 
                 songs_to_delete.len(), 
                 albums_to_delete.len(), 
                 artists_to_delete.len());
    }

    println!("Successfully removed folder '{}'", id);
}

fn parse_duration_to_seconds(duration_str: &str) -> Result<f64, String> {
    let parts: Vec<&str> = duration_str.split(':').collect();

    if parts.len() != 3 {
        return Err(format!("invalid format: '{}'. expected HH:MM:SS.ms", duration_str));
    }

    let hours = parts[0]
        .parse::<u64>()
        .map_err(|e| format!("invalid hours part in '{}': {}", duration_str, e))?;

    let minutes = parts[1]
        .parse::<u64>()
        .map_err(|e| format!("invalid minutes part in '{}': {}", duration_str, e))?;

    let seconds_ms_parts: Vec<&str> = parts[2].split('.').collect();

    let seconds = seconds_ms_parts[0]
        .parse::<u64>()
        .map_err(|e| format!("invalid seconds part in '{}': {}", duration_str, e))?;

    let mut milliseconds: f64 = 0.0;
    if seconds_ms_parts.len() > 1 && !seconds_ms_parts[1].is_empty() {
        let ms_str = seconds_ms_parts[1];
        let ms_value = ms_str
            .parse::<f64>()
            .map_err(|e| format!("Invalid milliseconds part in '{}': {}", duration_str, e))?;
        
        milliseconds = ms_value / (10.0f64.powi(ms_str.len() as i32));
    }

    //calculating the seconds needed
    let total_seconds = (hours as f64 * 3600.0) +
                        (minutes as f64 * 60.0) +
                        (seconds as f64) +
                        milliseconds;

    Ok(total_seconds)
}
