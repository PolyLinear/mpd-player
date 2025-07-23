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
}
