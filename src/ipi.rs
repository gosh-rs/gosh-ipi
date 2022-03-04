// [[file:../ipi.note::ac2d8efb][ac2d8efb]]
use super::*;
use socket::*;

use gosh_model::*;
use std::path::{Path, PathBuf};

use futures::SinkExt;
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};
use tokio_util::codec::Decoder;
use tokio_util::codec::{FramedRead, FramedWrite};
// ac2d8efb ends here

// [[file:../ipi.note::d4f83f32][d4f83f32]]
impl Computed {
    fn from_model_properties(mp: &ModelProperties) -> Self {
        let energy = dbg!(mp.get_energy().unwrap());
        let forces = mp.get_forces().unwrap().clone();
        Self {
            energy,
            forces,
            // TODO: we have no support for stress tensor, so set virial as
            // zeros
            virial: [0.0; 9],
            extra: "".into(),
        }
    }
}
// d4f83f32 ends here

// [[file:../ipi.note::104ce11f][104ce11f]]
/// The communication between the i-PI client and server.
struct IpiServerStream<R, W> {
    read: FramedRead<R, codec::ClientCodec>,
    write: FramedWrite<W, codec::ServerCodec>,
}

impl<R: AsyncRead + std::marker::Unpin, W: AsyncWrite + std::marker::Unpin> IpiServerStream<R, W> {
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

// [[file:../ipi.note::7804b9ff][7804b9ff]]
async fn ipi_client_loop<R, W>(mut bbm: BlackBoxModel, mol_ini: Molecule, read: R, write: W) -> Result<()>
where
    R: AsyncRead + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    // for the message we received from the server (the driver)
    let mut server_read = FramedRead::new(read, codec::ServerCodec);
    // for the message we sent to the server (the driver)
    let mut client_write = FramedWrite::new(write, codec::ClientCodec);

    let mut mol_to_compute: Option<Molecule> = None;
    let mut f_init = false;
    // NOTE: There is no async for loop for stream in current version of Rust,
    // so we use while loop instead
    while let Some(stream) = server_read.next().await {
        let mut stream = stream?;
        match stream {
            ServerMessage::Status => {
                debug!("server ask for client status");
                if !f_init {
                    client_write.send(ClientMessage::Status(ClientStatus::NeedInit)).await?;
                } else if mol_to_compute.is_some() {
                    client_write.send(ClientMessage::Status(ClientStatus::HaveData)).await?;
                } else {
                    client_write.send(ClientMessage::Status(ClientStatus::Ready)).await?;
                }
            }
            // initialization
            ServerMessage::Init(data) => {
                // FIXME: initialize data
                debug!("server sent init data: {:?}", data);
                f_init = true;
            }
            // receives structural information
            ServerMessage::PosData(mol) => {
                debug!("server sent mol {:?}", mol);
                mol_to_compute = Some(mol);
            }
            ServerMessage::GetForce => {
                debug!("server asks for forces");
                if let Some(mol) = mol_to_compute.as_mut() {
                    assert_eq!(mol.natoms(), mol_ini.natoms());
                    // NOTE: reset element symbols from mol_ini
                    mol.set_symbols(mol_ini.symbols());
                    let mp = bbm.compute(&mol)?;
                    let computed = Computed::from_model_properties(&mp);
                    client_write.send(ClientMessage::ForceReady(computed)).await?;
                    mol_to_compute = None;
                } else {
                    bail!("not mol to compute!");
                }
            }
            ServerMessage::Exit => {
                debug!("Received exit message from the server. Bye bye!");
                break;
            }
        }
    }

    Ok(())
}
// 7804b9ff ends here

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

// [[file:../ipi.note::ac221478][ac221478]]
pub async fn ipi_client(mut bbm: BlackBoxModel, mol_ini: Molecule, stream: IpiStream) -> Result<()> {
    match stream {
        IpiStream::Tcp(mut s) => {
            let (read, write) = s.split();
            ipi_client_loop(bbm, mol_ini, read, write).await?;
        }
        IpiStream::Unix(mut s) => {
            let (read, write) = s.split();
            ipi_client_loop(bbm, mol_ini, read, write).await?;
        }
    };

    Ok(())
}
// ac221478 ends here

// [[file:../ipi.note::680b1817][680b1817]]
use task::RxInput;

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
    /// Serve molecule computation reqeusts from channel `rx_inp`
    pub async fn serve_channel(&self, rx_inp: &mut RxInput) -> Result<()> {
        info!("i-PI server: wait for external code connection and incoming molecule to compute ...");
        let mut client_stream = self.accept().await?;
        client_stream.wait_until_ready().await?;
        debug!("client is ready now ...");

        loop {
            debug!("wait for new molecule to compute ...");
            let (mol, tx_out) = rx_inp.recv().await.ok_or(format_err!("mol channel dropped"))?;
            debug!("ask client to compute molecule {}", mol.title());
            let computed = client_stream.compute_one(mol).await?;
            match tx_out.send(computed) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        Ok(())
    }
}
// 680b1817 ends here
