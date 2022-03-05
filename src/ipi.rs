// [[file:../ipi.note::ac2d8efb][ac2d8efb]]
use super::*;
use socket::*;

use futures::SinkExt;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};
use tokio_util::codec::Decoder;
use tokio_util::codec::{FramedRead, FramedWrite};
// ac2d8efb ends here

// [[file:../ipi.note::104ce11f][104ce11f]]
/// The communication between the i-PI client and server.
struct IpiServerStream<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    read: FramedRead<R, codec::ClientCodec>,
    write: FramedWrite<W, codec::ServerCodec>,
}

impl<R, W> IpiServerStream<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn new(read: R, write: W) -> Self {
        // the message we received from the client code (VASP, SIESTA, ...)
        let mut read = FramedRead::new(read, codec::ClientCodec);
        // the message we sent to the client
        let mut write = FramedWrite::new(write, codec::ServerCodec);

        Self { read, write }
    }

    /// Ask and return client status
    async fn get_status(&mut self) -> Result<ClientStatus> {
        self.write.send(ServerMessage::Status).await?;
        let stream = self.read.next().await.ok_or(format_err!("client stream"))?;
        match stream? {
            ClientMessage::Status(status) => Ok(status),
            x => bail!("Inconsistent client state: {x:?}"),
        }
    }

    /// Send an exit message to client to let them exit gracefully.
    async fn set_exit(&mut self) -> Result<()> {
        self.write.send(ServerMessage::Exit).await?;
        Ok(())
    }

    /// Get computed results (potential energy, force and virial) from the
    /// client
    async fn get_computed(&mut self) -> Result<Computed> {
        self.write.send(ServerMessage::GetForce).await?;
        let stream = self.read.next().await.ok_or(format_err!("client stream"))?;
        match stream? {
            ClientMessage::ForceReady(computed) => Ok(computed),
            _ => bail!("Inconsistent client state"),
        }
    }

    /// Send input data (the position and cell data) to the client.
    async fn set_input(&mut self, mol: Molecule) -> Result<()> {
        self.write.send(ServerMessage::PosData(mol)).await?;
        Ok(())
    }

    /// Send the init string to the client.
    // FIXME: handle the json part
    async fn set_init(&mut self) -> Result<()> {
        let init = InitData::new(0, "");
        self.write.send(ServerMessage::Init(init)).await?;
        Ok(())
    }
}
// 104ce11f ends here

// [[file:../ipi.note::32f96fbd][32f96fbd]]
// wait until client ready to compute molecule
macro_rules! process_client_stream {
    ($stream: expr) => {{
        let (read, write) = $stream.split();
        let mut stream = IpiServerStream::new(read, write);

        loop {
            // ask for client status
            let status = stream.get_status().await?;
            match status {
                ClientStatus::NeedInit => {
                    stream.set_init().await?;
                }
                ClientStatus::Ready => {
                    break;
                }
                _ => unimplemented!(),
            }
        }
    }};
}

// compute one molecule, and return computed properties
macro_rules! process_client_stream_compute {
    ($stream: expr, $mol: expr) => {{
        let (read, write) = $stream.split();
        let mut stream = IpiServerStream::new(read, write);

        // client is ready, and we send the mol to compute
        stream.set_input($mol).await?;
        let status = stream.get_status().await?;
        assert_eq!(status, ClientStatus::HaveData);

        let computed = stream.get_computed().await?;
        return Ok(computed);
    }};
}
// 32f96fbd ends here

// [[file:../ipi.note::680b1817][680b1817]]
use task::TaskReceiver;

impl IpiStream {
    async fn wait_until_ready(&mut self) -> Result<()> {
        match self {
            IpiStream::Tcp(s) => {
                process_client_stream!(s);
            }
            IpiStream::Unix(s) => {
                process_client_stream!(s);
            }
        };

        Ok(())
    }

    async fn compute_one(&mut self, mol: Molecule) -> Result<Computed> {
        let computed = match self {
            IpiStream::Tcp(s) => {
                process_client_stream_compute!(s, mol);
            }
            IpiStream::Unix(s) => {
                process_client_stream_compute!(s, mol);
            }
        };
        Ok(computed)
    }
}

impl IpiListener {
    /// Serve molecule computation reqeusts from `task`
    pub async fn serve_channel(&self, task: &mut TaskReceiver) -> Result<()> {
        info!("i-PI server: wait for external code connection and incoming molecule to compute ...");
        let mut client_stream = self.accept().await?;
        client_stream.wait_until_ready().await?;
        debug!("client is ready now ...");

        loop {
            debug!("wait for new molecule to compute ...");
            if let Some((mol, tx_out)) = task.recv().await {
                debug!("ask client to compute molecule {}", mol.title());
                let computed = client_stream.compute_one(mol).await?;
                match tx_out.send(computed) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            } else {
                // task channel closed for some reason
                client_stream.shutdown().await;
                break;
            }
        }

        Ok(())
    }
}
// 680b1817 ends here

// [[file:../ipi.note::1b623d31][1b623d31]]
macro_rules! process_client_stream_exit {
    ($stream: expr) => {{
        let (read, write) = $stream.split();
        let _ = IpiServerStream::new(read, write).set_exit().await;
    }};
}

impl IpiStream {
    async fn shutdown(&mut self) {
        info!("sent exit message to client");
        match self {
            IpiStream::Tcp(s) => {
                process_client_stream_exit!(s);
            }
            IpiStream::Unix(s) => {
                process_client_stream_exit!(s);
            }
        };
    }
}
// 1b623d31 ends here
