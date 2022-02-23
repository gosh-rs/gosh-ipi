// [[file:../ipi.note::ac2d8efb][ac2d8efb]]
use super::*;
use std::path::{Path, PathBuf};

use gosh_model::*;
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

// [[file:../ipi.note::624a82ac][624a82ac]]
/// Guess the unix socket file name from host name for the i-PI server.
fn guess_unix_socket_file(host: &str) -> String {
    format!("/tmp/ipi_{host}")
}
// 624a82ac ends here

// [[file:../ipi.note::ac221478][ac221478]]
pub async fn bbm_as_ipi_client(mut bbm: BlackBoxModel, mol_ini: Molecule, sock: &std::path::Path) -> Result<()> {
    use futures::SinkExt;
    use futures::StreamExt;
    use tokio::net::UnixStream;
    use tokio_util::codec::{FramedRead, FramedWrite};

    // let mut stream = UnixStream::connect(sock).context("connect to unix socket").await?;
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:10244")
        .await
        .context("connect to host")?;
    let (read, write) = stream.split();

    // for the message we received from the server (the driver)
    let mut server_read = FramedRead::new(read, codec::ServerCodec);
    // for the message we sent to the server (the driver)
    let mut client_write = FramedWrite::new(write, codec::ClientCodec);

    let mut mol_to_compute: Option<Molecule> = None;
    // NOTE: There is no async for loop for stream in current version of Rust,
    // so we use while loop instead
    while let Some(stream) = server_read.next().await {
        let mut stream = stream?;
        match stream {
            ServerMessage::Status => {
                debug!("server ask for client status");
                if mol_to_compute.is_none() {
                    client_write.send(ClientMessage::Status(ClientStatus::Ready)).await?;
                } else {
                    client_write.send(ClientMessage::Status(ClientStatus::HaveData)).await?;
                }
            }
            ServerMessage::GetForce => {
                debug!("server ask for forces");
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
            ServerMessage::PosData(mol) => {
                debug!("server sent mol {:?}", mol);
                mol_to_compute = Some(mol);
            }
            ServerMessage::Init(data) => {
                debug!("server sent init data: {:?}", data);
            }
            ServerMessage::Exit => {
                debug!("server ask exit");
                break;
            }
        }
    }

    Ok(())
}
// ac221478 ends here

// [[file:../ipi.note::77afd524][77afd524]]
pub async fn ipi_server(sock: &Path, mol: &Molecule) -> Result<()> {
    use futures::SinkExt;
    use futures::StreamExt;
    use tokio::net::UnixListener;
    use tokio_util::codec::{FramedRead, FramedWrite};

    let mut listener = UnixListener::bind(sock).context("bind unix socket")?;
    let (mut stream, _) = listener.accept().await.context("accept new unix socket client")?;
    let (read, write) = stream.split();

    // the message we received from the client code (VASP, SIESTA, ...)
    let mut client_read = FramedRead::new(read, codec::ClientCodec);
    // the message we sent to the client
    let mut server_write = FramedWrite::new(write, codec::ServerCodec);

    loop {
        // ask for client status
        server_write.send(ServerMessage::Status).await?;
        // read the message
        if let Some(stream) = client_read.next().await {
            let stream = stream?;
            match stream {
                // we are ready to send structure to compute
                ClientMessage::Status(status) => match status {
                    ClientStatus::Ready => {
                        server_write.send(ServerMessage::PosData(mol.clone())).await?;
                    }
                    ClientStatus::NeedInit => {
                        let init = InitData::new(0, "");
                        server_write.send(ServerMessage::Init(init)).await?;
                    }
                    ClientStatus::HaveData => {
                        server_write.send(ServerMessage::GetForce).await?;
                    }
                },
                // the computation is done, and we got the results
                ClientMessage::ForceReady(computed) => {
                    dbg!(computed);
                    break;
                }
            }
        }
    }
    Ok(())
}
// 77afd524 ends here
