use std::{cell::RefCell, collections::{HashSet, VecDeque}, fs, path::{Path, PathBuf}, rc::Rc};
use audiotags::{Tag, Picture};

use crate::core::song::{Song, Image, Album};

#[derive(Debug)]
pub enum CError {
    InvalidTag(PathBuf),
    InvalidPath,
}

fn find_or_create_album(albums: &mut Vec<Rc<Album>>, title: &str, artists: &[String], cover: Option<Image>) -> Rc<Album> {
    // Check if we already have this album
    for album in albums.iter() {
        if album.title == title && 
           album.artists.len() == artists.len() && 
           album.artists.iter().zip(artists.iter()).all(|(a, b)| a == b) {
            return Rc::clone(album);
        }
    }

    let album = Rc::new(Album {
        title: title.to_string(),
        artists: artists.to_vec(),
        songs: RefCell::new(Vec::new()),
        cover,
    });

    albums.push(Rc::clone(&album));
    album
}

pub fn parse_file<P: AsRef<Path>>(path: P, albums: &mut Vec<Rc<Album>>) -> Result<Rc<Song>, CError> {
    let tag = match Tag::new().read_from_path(&path) {
        Ok(t) => t,
        Err(e) => {
            println!("parse_file() tag error for {}: {e}", path.as_ref().display());
            return Err(CError::InvalidTag(path.as_ref().to_path_buf()));
        }
    };
    
    let title = tag.title().unwrap_or("unknown song");
    let artist = tag.artist().unwrap_or(title);
    let album_title = tag.album_title().unwrap_or("unknown album");
    
    let album_artists = match tag.album_artists() {
        Some(a) => a.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
        None => vec![artist.to_string()],
    };
    
    let cover = tag.album_cover().map(|img| img.into());

    let album = find_or_create_album(albums, album_title, &album_artists, cover);
    
    let features = if let Some(mut artists_list) = tag.artists() {
        artists_list.retain(|a| *a != artist);
        if artists_list.is_empty() {
            None
        } else {
            Some(artists_list.iter().map(|a| a.to_string()).collect())
        }
    } else {
        None
    };
    
    let track_num = tag.track_number().unwrap_or(1);
    let disc_num = tag.disc_number().unwrap_or(1);

    let song = Rc::new(Song {
        title: title.to_string(),
        artist: artist.to_string(),
        album: Rc::clone(&album),
        features,
        track_num,
        disc_num,
        cover: None,
        path: path.as_ref().to_path_buf(),
    });

    album.songs.borrow_mut().push(Rc::downgrade(&song));
    
    Ok(song)
}

pub fn scan_dir<P: AsRef<Path>>(dir: P, songs: &mut Vec<Rc<Song>>, albums: &mut Vec<Rc<Album>>) {
    let mut dir_queue = VecDeque::new();
    dir_queue.push_back(dir.as_ref().to_path_buf());
    
    while let Some(current_dir) = dir_queue.pop_front() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(err) => {
                println!("Failed to read directory {}: {}", current_dir.display(), err);
                continue;
            }
        };
        
        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    println!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();
            
            if path.is_file() {
                match parse_file(&path, albums) {
                    Ok(song) => songs.push(song),
                    Err(e) => {
                        println!("Metadata extraction failed for {}: {:?}", path.display(), e);
                    }
                }
            } else if path.is_dir() {
                dir_queue.push_back(path);
            } else {
                println!("Unsupported file type: {}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dir_test() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("music");

        let mut songs = Vec::new();
        let mut albums = Vec::new();

        scan_dir(&path, &mut songs, &mut albums);
        
        println!("Found {} songs in {} albums", songs.len(), albums.len());

        for s in songs {
            println!("Song: {}", s.title);
        }

        for a in albums {
            println!("Album: {} by {:?} with {} songs", 
                a.title, 
                a.artists, 
                a.songs.borrow().len());
        }
    }
}