use std::net::TcpStream;
use std::io;
use std::time::Duration;
use thiserror::Error;

// --- Custom Error Type ---
// Provides specific information about connection failures.
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

// --- Configuration Constants ---
const CONNECTION_HOST: &str = "127.0.0.1";
const CONNECTION_RETRIES: u32 = 5;
const RETRY_DELAY: Duration = Duration::from_millis(500);

/// Attempts to connect to the specified TCP port on localhost.
///
/// Implements a retry mechanism to handle cases where the server might not be
/// immediately available after being started.
pub fn connect(port: u16) -> Result<TcpStream, ConnectionError> {
    let address = format!("{}:{}", CONNECTION_HOST, port);
    println!("Attempting to connect to scrcpy server at {}...", address);

    for attempt in 0..CONNECTION_RETRIES {
        match TcpStream::connect(&address) {
            Ok(stream) => {
                println!("Successfully connected to scrcpy server.");
                // Set TCP_NODELAY for low-latency streaming. This is a common
                // optimization for applications like this.
                if let Err(e) = stream.set_nodelay(true) {
                    eprintln!("Warning: Failed to set TCP_NODELAY: {}", e);
                }
                return Ok(stream);
            }
            Err(e) => {
                if attempt == CONNECTION_RETRIES - 1 {
                    // Last attempt failed, return the final error
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
    // This code is logically unreachable because the loop above will always return.
    // The `unreachable!` macro informs the compiler of this.
    unreachable!();
}

