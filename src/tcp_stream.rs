use std::net::TcpStream;
use std::io;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to connect to {address} after {retries} retries.")]
    ConnectionFailed {
        address: String,
        retries: u32,
        #[source]
        source: io::Error,
    },
}

const CONNECTION_HOST: &str = "127.0.0.1";
const CONNECTION_RETRIES: u32 = 5;
const RETRY_DELAY: Duration = Duration::from_millis(500);

pub fn connect(port: u16) -> Result<TcpStream, ConnectionError> {
    let address = format!("{}:{}", CONNECTION_HOST, port);
    println!("Attempting to connect to scrcpy server at {}...", address);

    for attempt in 0..CONNECTION_RETRIES {
        match TcpStream::connect(&address) {
            Ok(stream) => {
                println!("Successfully connected to scrcpy server.");
                let _ = stream.set_nodelay(true);
                return Ok(stream);
            }
            Err(e) => {
                if attempt == CONNECTION_RETRIES - 1 {
                    return Err(ConnectionError::ConnectionFailed {
                        address,
                        retries: CONNECTION_RETRIES,
                        source: e,
                    });
                }
                println!(
                    "Connection attempt {}/{} failed. Retrying in {:?}...",
                    attempt + 1,
                    CONNECTION_RETRIES,
                    RETRY_DELAY
                );
                std::thread::sleep(RETRY_DELAY);
            }
        }
    }
    unreachable!();
}
