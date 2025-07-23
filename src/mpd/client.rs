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

    //TODO: checking errors is going to be repeated often; implement generic error handling across
    //commands
    pub fn get_playlist(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        self.write_con.write(b"playlist\n")?;
        self.write_con.flush()?;

        let mut iter = self.read_con.by_ref().lines().flatten();
        if let Some(first) = iter.next() {
            if first.starts_with("ACK [") {
                return Err("Command was not executed correctly".into());
            }
            let mut result = Vec::with_capacity(16);
            result.push(first);
            result.extend(iter.take_while(|line| line != "OK"));
            return Ok(result);
        }

        Err("Timeout".into())
    }
}
