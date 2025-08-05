use rodio::{OutputStream, Sink};
use std::path::Path;
use std::fs::File;
use std::sync::{Arc, mpsc};
use std::thread;
use crate::core::song::Song;

pub struct CPlayer {
    sink: Sink,
    _stream: OutputStream,
    queue: Vec<Song>,
    pos: usize,

}


#[derive(PartialEq)]
pub enum AudioCommand {
    PlayNow(Song),
    Pause,
    Play,
    TogglePlay,
    QueueNext(Song),
    QueueEnd(Song),
    AckCommand(String),
}


pub struct PlayerController {
    command_sender: mpsc::Sender<AudioCommand>,
    command_receiver: mpsc::Receiver<AudioCommand>,
}

impl CPlayer {
    pub fn new() -> Self {             
        let stream = rodio::OutputStreamBuilder::open_default_stream().unwrap();

        CPlayer {
            sink: rodio::Sink::connect_new(stream.mixer()),
            _stream: stream,
            queue: Vec::new(),
            pos: 0
        }
    }

    pub fn play_now(&mut self, song: Song) {
        if self.pos >= self.queue.len() {
            self.queue.push(song.clone());
        }

        else{
            self.queue.insert(self.pos + 1, song.clone());
            self.pos += 1;
        }

        let song_file = match File::open(&song.path) {
            Ok(s) => s,
            Err(e) => {
                todo!();
                println!("{e}");
                return;
            },
        };
        
        //here i can just clear the sink and play the new song
        self.sink.clear();
        
        if let Ok(d) = rodio::Decoder::try_from(song_file) {
            self.sink.append(d);
        }

        self.sink.play();
        
    }

    pub fn pause_song(&self) {
        self.sink.pause();
    }

    pub fn play_song(&self) {
        self.sink.play();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn queue_next(&mut self, song: Song) {
        if self.pos < self.queue.len()-1 {
            self.queue.insert(self.pos+1, song.clone());
        }
        else{
            self.queue.push(song);
        }
    }

    pub fn queue_end(&mut self, song: Song) {
        self.queue.push(song)
    }
    
}

// sends messsages to and 
impl PlayerController {
    pub fn new(sender: mpsc::Sender<AudioCommand>, receiver: mpsc::Receiver<AudioCommand>) -> Self {
        PlayerController {
            command_sender: sender,
            command_receiver: receiver,
        }
    }

    pub fn play_now(&self, song: Song) {
        self.command_sender
            .send(AudioCommand::PlayNow(song))
            .expect("Audio thread has panicked and disconnected.");
    }

    pub fn pause(&self) {
        self.command_sender
            .send(AudioCommand::Pause)
            .expect("Audio thread has panicked and disconnected.");
    }

    pub fn play(&self) {
        self.command_sender
            .send(AudioCommand::Play)
            .expect("Audio thread has panicked and disconnected.");
    }

    pub fn queue_next(&self, song: Song) {
        self.command_sender
            .send(AudioCommand::QueueNext(song))
            .expect("Audio thread has panicked and disconnected.");
    }

    pub fn queue_end(&self, song: Song) {
        self.command_sender
            .send(AudioCommand::QueueEnd(song))
            .expect("Audio thread has panicked and disconnected.");
    }

    pub fn toggle_play(&self) {
        self.command_sender
            .send(AudioCommand::TogglePlay)
            .expect("Audio thread has panicked and disconnected");
    }
}

pub fn audio_thread_loop(receiver: mpsc::Receiver<AudioCommand>, sender: mpsc::Sender<AudioCommand>) {
    let mut player = CPlayer::new();
    
    sender.send(AudioCommand::AckCommand("audio thread started".into())).unwrap();

    for command in receiver {

        match command {
            AudioCommand::PlayNow(song) => player.play_now(song),
            AudioCommand::Pause => player.pause_song(),
            AudioCommand::Play => player.play_song(),
            AudioCommand::QueueNext(song) => player.queue_next(song),
            AudioCommand::QueueEnd(song) => player.queue_end(song),
            AudioCommand::TogglePlay => {
                if player.is_paused() {
                    player.play_song();
                }
                else {
                    player.pause_song();
                }
            }
            _ => {},
        }
    }
    println!("audio thread ended");
}


mod test {
    use tauri::Manager;
    use crate::MusicLibrary;
    use std::sync::Mutex;

    use crate::AppState;

    use super::AudioCommand;


    #[test]
    fn test_thread_msg() {
        let app = tauri::test::mock_app();
        app.manage(Mutex::new(MusicLibrary::new()));

        let expected = AudioCommand::AckCommand("audio thread started".into());
        let app_state = app.state::<AppState>();
        let state = app_state.lock().unwrap();

        assert!(expected == state.player.command_receiver.recv().unwrap(), 
            "audio thread did not send correct ack command");
    }
    
}
