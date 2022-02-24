// [[file:../ipi.note::14beb047][14beb047]]
use super::*;

use tokio::net::{TcpStream, UnixStream};
use tokio_util::codec::{FramedRead, FramedWrite};
// 14beb047 ends here

// [[file:../ipi.note::624a82ac][624a82ac]]
/// Guess the unix socket file name from host name for the i-PI server.
fn guess_unix_socket_file(host: &str) -> String {
    format!("/tmp/ipi_{host}")
}
// 624a82ac ends here

// [[file:../ipi.note::9b4b9ee0][9b4b9ee0]]
#[derive(Debug)]
pub enum Socket {
    Tcp(TcpStream),

    #[cfg(unix)]
    Unix(UnixStream),
}

impl Socket {
    /// Connect to unix socket or internet socket.
    pub async fn connect(host: &str, port: u16, unix: bool) -> Result<Self> {
        let socket = if unix {
            let sock_file = guess_unix_socket_file(host);
            debug!("connect to unix domain socket: {sock_file}");
            let stream = UnixStream::connect(sock_file).await.context("connect to uds")?;
            Self::Unix(stream)
        } else {
            debug!("connect to socket {host}:{port}");
            let stream = TcpStream::connect((host, port)).await.context("connect to inet")?;
            Self::Tcp(stream)
        };
        Ok(socket)
    }
}
// 9b4b9ee0 ends here
