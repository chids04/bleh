use audiotags::Picture;
use core::fmt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, Weak};
use uuid::Uuid;

use serde::{Serialize, Deserialize};

use std::cell::RefCell;

//tauri requires
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artist: Uuid,
    pub album: Uuid,
    pub features: Option<Vec<(Option<Uuid>, String)>>,
    pub track_num: u16,
    pub disc_num: u16,
    pub cover: Option<Image>,
    pub path: PathBuf,
    pub duration: f64,
    pub folder_id: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Artist {
    pub id: Uuid,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ArtistType {
    KnownArtist(Uuid),
    UnknownArtist(String)

}

impl ArtistType {
    pub fn name(&self) -> &str {
        match self {
            ArtistType::KnownArtist(_) => {
                "Known Artist" // TODO: implement proper lookup
            }
            ArtistType::UnknownArtist(name) => name
        }
    }
    
    pub fn has_profile(&self) -> bool {
        matches!(self, ArtistType::KnownArtist(_))
    }
    
    pub fn promote_to_known(&self, new_id: Uuid) -> ArtistType {
        ArtistType::KnownArtist(new_id)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Album {
    pub id: Uuid,
    pub artists: Vec<(Option<Uuid>, String)>,
    pub cover: Option<Image>,
    pub title: String,
    pub songs: Vec<Uuid>,
}


#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct Image {
    pub data: Vec<u8>,
    pub extension: String,
}


impl PartialEq for Album {
    //only need to compare the artists and album artists
    fn eq(&self, other: &Self) -> bool {
    self.title == other.title &&
        self.artists == other.artists
    }
}

impl fmt::Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cover = match &self.cover {
            Some(cover) => &cover.extension,
            None => "none",
        };

        writeln!(
            f,
            "album title: {}\nalbum artists {:#?}\nimage: {cover}",
            self.title, self.artists
        )?;

        for s in &self.songs {
            writeln!(f, "song with id: {s}");
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
        } else {
            writeln!(f, "individual_cover: none")?;
        }

        writeln!(f, "path: {:#?}", self.path)
    }
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

#[derive(Clone, Serialize)]
pub struct SongDto {
    pub title: String,
    pub artist: String,
    pub features: Option<Vec<u64>>,
    pub track_num: u16,
    pub disc_num: u16,
    pub path: PathBuf,
    pub cover: Option<Image>,
    pub album: AlbumDto, 
}

#[derive(Clone, Serialize)]
pub struct AlbumDto {
    pub title: String,
    pub artists: Vec<String>,
    pub cover: Option<Image>,
}

