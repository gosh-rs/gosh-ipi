// [[file:../ipi.note::14beb047][14beb047]]
use super::*;

use tokio::net::{TcpListener, UnixListener};
use tokio::net::{TcpStream, UnixStream};
use tokio_util::codec::{FramedRead, FramedWrite};
// 14beb047 ends here

// [[file:../ipi.note::624a82ac][624a82ac]]
/// A socket for i-PI client or driver
#[derive(Debug, Clone)]
pub struct Socket {}

/// Guess the unix socket file name from host name for the i-PI server.
fn guess_unix_socket_file(host: &str) -> String {
    format!("/tmp/ipi_{host}")
}
// 624a82ac ends here

// [[file:../ipi.note::2d2abd6a][2d2abd6a]]
/// Return the address available for binding with the OS assigns port.
pub fn get_free_tcp_address() -> Option<std::net::SocketAddr> {
    std::net::TcpListener::bind(("localhost", 0)).ok()?.local_addr().ok()
}
// 2d2abd6a ends here

// [[file:../ipi.note::9b4b9ee0][9b4b9ee0]]
#[derive(Debug)]
/// A stream between i-PI client and driver (server)
pub enum IpiStream {
    Tcp(TcpStream),

    #[cfg(unix)]
    Unix(UnixStream),
}

impl Socket {
    /// Opens i-PI connection to a driver.
    pub async fn connect(host: &str, port: u16, unix: bool) -> Result<IpiStream> {
        let stream = if unix {
            let sock_file = guess_unix_socket_file(host);
            debug!("connect to unix domain socket: {sock_file}");
            let stream = UnixStream::connect(sock_file).await.context("connect to uds")?;
            IpiStream::Unix(stream)
        } else {
            debug!("connecting to socket {host}:{port}");
            let stream = TcpStream::connect((host, port)).await.context("connect to inet")?;
            IpiStream::Tcp(stream)
        };
        Ok(stream)
    }
}
// 9b4b9ee0 ends here

// [[file:../ipi.note::ad23dfbd][ad23dfbd]]
#[derive(Debug)]
/// The listener for servier side
pub enum IpiListener {
    Tcp(TcpListener),

    #[cfg(unix)]
    Unix(UnixListener),
}

impl Socket {
    /// Listening on incoming connections using unix socket or internet socket.
    pub async fn bind(host: &str, port: u16, unix: bool) -> Result<IpiListener> {
        let x = if unix {
            let sock_file = guess_unix_socket_file(host);
            debug!("listening on unix domain socket: {sock_file}");
            let listener = UnixListener::bind(sock_file).context("binding on uds")?;
            IpiListener::Unix(listener)
        } else {
            debug!("listening on {host}:{port}");
            let listener = TcpListener::bind((host, port)).await.context("binding on inet")?;
            IpiListener::Tcp(listener)
        };

        Ok(x)
    }
}

impl IpiListener {
    /// Accepts a new incoming connection from this listener.
    pub async fn accept(&self) -> Result<IpiStream> {
        let s = match self {
            Self::Tcp(l) => {
                let (s, _) = l.accept().await?;
                IpiStream::Tcp(s)
            }
            Self::Unix(l) => {
                let (s, _) = l.accept().await?;
                IpiStream::Unix(s)
            }
        };
        Ok(s)
    }
}
// ad23dfbd ends here
