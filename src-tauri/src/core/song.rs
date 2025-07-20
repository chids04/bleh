
use core::fmt;
use std::path::PathBuf;
use audiotags::Picture;

use std::rc::{Rc, Weak};
use std::cell::RefCell;


pub struct Song{
    pub title: String,
    pub artist: String,
    pub album: Rc<Album>,
    pub features: Option<Vec<String>>,
    pub track_num: u16,
    pub disc_num: u16,
    pub cover: Option<Image>,
    pub path: PathBuf,
}

pub struct Album {
    pub artists: Vec<String>,
    pub cover: Option<Image>,
    pub title: String,
    pub songs: RefCell<Vec<Weak<Song>>>,
}

impl fmt::Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let cover = match &self.cover {
            Some(cover) => &cover.extension,
            None => "none",
        };

        writeln!(f, "album title: {}\nalbum artists {:#?}\nimage: {cover}", 
            self.title, self.artists)?;
        
        for song_weak in self.songs.borrow().iter() {
            if let Some(song) = song_weak.upgrade() {
                writeln!(f, "{}", song.title)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "title: {}", self.title)?;
        writeln!(f, "artist: {}", self.artist)?;
        writeln!(f, "album: {}", self.album)?;

        if let Some(feats) = &self.features {
            writeln!(f, "features {:#?}", feats)?;
        } 

        writeln!(f, "track_num: {}", self.track_num)?;
        writeln!(f, "disc_num: {}", self.disc_num)?;

        if let Some(i) = &self.cover {
            writeln!(f, "song_cover: {} image", i.extension)?;
        }
        else{
            writeln!(f, "album_cover: none")?;
        }

        writeln!(f,"path: {:#?}", self.path)
        
    }
}

#[derive(PartialEq, Eq)]
pub struct Image {
    pub data: Vec<u8>,
    pub extension: String,
}

impl From<Picture<'_>> for Image {
    fn from(pic: Picture) -> Self {
        let extension: String = pic.mime_type.into();
        let extension = extension.to_lowercase();

        Image {
            data: pic.data.to_vec(),
            extension,
        }
    }
}

