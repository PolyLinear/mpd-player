#![allow(unused)]
use std::{
    error::Error,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    time::Duration,
};

pub struct MPDClient {
    read_con: BufReader<TcpStream>,
    write_con: BufWriter<TcpStream>,
}

#[derive(Debug)]
pub enum MPDError {
    FailedWrite,
    FailedCommand,
}
macro_rules! lines {
    ($client:expr) => {
        $client
            .read_con
            .by_ref()
            .lines()
            .flatten()
            .take_while(|line| line != "OK")
    };
}

pub enum MPDQueueState {
    Off,
    On,
    Oneshot,
}

impl Default for MPDQueueState {
    fn default() -> Self {
        Self::Off
    }
}

pub enum MPDState {
    Play,
    Stop,
    Pause,
}

impl Default for MPDState {
    fn default() -> Self {
        Self::Stop
    }
}

#[derive(Default)]
pub struct MPDStatus {
    partition: Option<String>,
    volume: Option<u8>,
    repeat: MPDQueueState,
    random: MPDQueueState,
    single: MPDQueueState,
    consume: MPDQueueState,
    playlist: Option<u32>,
    playlistlength: Option<u64>,
    state: MPDState,
    song: Option<u32>,
    songid: Option<u32>,
    nextsong: Option<u32>,
    nextsongid: Option<u32>,
    time: Option<u32>, //In format num:num docs don't tell me what the numbes represent
    elapsed: Option<f32>,
    duration: Option<f32>,
    bitrate: Option<u16>,
    xfade: Option<u16>,
    mixramdb: Option<u16>,
    mixrampdelay: Option<u16>,
    audio: Option<String>, //TODO samplerate:bits:channels (need to further format this string)
    //update_db: Option<> not included for now
    error: Option<String>,
    lastloadedplaylist: Option<String>,
}

impl MPDStatus {
    fn new<T: Iterator>(iter: &mut T) -> Result<Self, MPDError> {
        let mut status = MPDStatus::default();
        todo!("Implement MPD STATUS");
        Ok(status)
    }
}

#[derive(Default)]
pub struct MPDStats {
    artists: u32,
    albums: u32,
    uptime: u32,
    db_playtime: u64,
    db_update: u64,
    playtime: u32,
}

impl MPDStats {
    fn new<T: Iterator>(iter: &mut T) -> Result<Self, MPDError> {
        let mut stats = MPDStats::default();
        todo!("Implement MPD STATS");
        Ok(stats)
    }
}

impl std::fmt::Display for MPDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedWrite => write!(f, "{}", "Failed to write to socket".to_string()),
            Self::FailedCommand => write!(f, "{}", "Failed to execute the command".to_string()),
        }
    }
}

impl Error for MPDError {}

impl MPDClient {
    pub fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Duration::from_secs(5).into())?;

        let mut read_con = BufReader::new(stream.try_clone()?);
        let write_con = BufWriter::new(stream);

        let mut buffer = [0u8; 20];
        read_con.read(&mut buffer)?;

        if !buffer.starts_with(b"OK MPD") {
            return Err("MPD OK was not received".into());
        }

        Ok(Self {
            read_con,
            write_con,
        })
    }

    fn mpd_command(&mut self, command: &[u8]) -> Result<String, MPDError> {
        self.write_con
            .write_all(command)
            .and_then(|_| self.write_con.flush())
            .map_err(|_| MPDError::FailedWrite)?;

        let mut buffer = String::new();
        self.read_con
            .read_line(&mut buffer)
            .map_err(|_| MPDError::FailedCommand)?;

        if buffer.starts_with("ACK [") {
            return Err(MPDError::FailedCommand);
        }

        Ok(buffer)
    }

    pub fn playlist(&mut self) -> Result<Vec<String>, MPDError> {
        let first = self.mpd_command(b"playlist\n")?;
        let mut playlist = Vec::with_capacity(16);
        playlist.push(first);
        playlist.extend(lines!(self));

        Ok(playlist)
    }

    pub fn list_playlist(&mut self, name: &str) -> Result<Vec<String>, MPDError> {
        let first = self.mpd_command(format!("listplaylist {}\n", name).as_bytes())?;
        let mut playlist = Vec::with_capacity(16);
        playlist.push(first);
        playlist.extend(lines!(self));

        Ok(playlist)
    }

    pub fn clear_queue(&mut self) -> Result<(), MPDError> {
        self.mpd_command(b"clear\n")?;
        Ok(())
    }

    pub fn load_playlist(&mut self, name: &str) -> Result<(), MPDError> {
        self.mpd_command(format!("load {}\n", name).as_bytes())?;
        Ok(())
    }

    pub fn get_status(&mut self) -> Result<Vec<String>, MPDError> {
        let first = self.mpd_command(b"status\n")?;
        let mut status_pairs = Vec::with_capacity(16);
        status_pairs.push(first);
        status_pairs.extend(lines!(self));

        Ok(status_pairs)
    }

    pub fn currentsong(&mut self) -> Result<Vec<String>, MPDError> {
        let _ = self.mpd_command(b"currentsong\n")?;
        let mut song_pairs = Vec::with_capacity(16);
        song_pairs.extend(lines!(self));
        Ok(song_pairs)
    }
}
