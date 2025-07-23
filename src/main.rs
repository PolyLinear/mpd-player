mod mpd;

use mpd::client::MPDClient;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    println!("{:#?}", MPDClient::new("127.0.0.1:6600")?.get_playlist()?);
    Ok(())
}
