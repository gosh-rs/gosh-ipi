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
        // the message we received from the client code (VASP, SIESTA, ...)
        let mut client_read = FramedRead::new(read, codec::ClientCodec);
        // the message we sent to the client
        let mut server_write = FramedWrite::new(write, codec::ServerCodec);

        loop {
            // ask for client status
            server_write.send(ServerMessage::Status).await?;
            // read the message, break if it is ready
            if let Some(stream) = client_read.next().await {
                match stream? {
                    // we are ready to send structure to compute
                    ClientMessage::Status(status) => match status {
                        ClientStatus::NeedInit => {
                            let init = InitData::new(0, "");
                            server_write.send(ServerMessage::Init(init)).await?;
                        }
                        ClientStatus::Ready => {
                            break;
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }
        }
    }};
}

// compute one molecule, and return computed properties
macro_rules! process_client_stream_compute {
    ($stream: expr, $mol: expr) => {{
        let (read, write) = $stream.split();
        // the message we received from the client code (VASP, SIESTA, ...)
        let mut client_read = FramedRead::new(read, codec::ClientCodec);
        // the message we sent to the client
        let mut server_write = FramedWrite::new(write, codec::ServerCodec);

        // client is ready, and we send the mol to compute
        server_write.send(ServerMessage::PosData($mol)).await?;
        server_write.send(ServerMessage::Status).await?;
        let stream = client_read.next().await.ok_or(format_err!("client stream"))?;
        match stream? {
            ClientMessage::Status(status) => {
                assert_eq!(status, ClientStatus::HaveData);
            }
            _ => unimplemented!(),
        }
        server_write.send(ServerMessage::GetForce).await?;
        let stream = client_read.next().await.ok_or(format_err!("client stream"))?;
        match stream? {
            ClientMessage::ForceReady(computed) => {
                return Ok(computed);
            }
            _ => unimplemented!(),
        }
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
