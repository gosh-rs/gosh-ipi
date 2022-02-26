// [[file:../ipi.note::ac2d8efb][ac2d8efb]]
use super::*;
use socket::Socket;

use gut::cli::*;
use gut::fs::*;
// ac2d8efb ends here

// [[file:../ipi.note::724c4c4c][724c4c4c]]
#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct IpiCli {
    #[clap(flatten)]
    verbose: Verbosity,

    /// path to bbm template
    #[clap(short = 't')]
    bbm: PathBuf,

    /// path to molecule to compute
    mol: PathBuf,

    /// The name of unix domain sock
    #[clap(short = 'u', default_value = "bbm-ipi.sock")]
    sock: String,
}
// 724c4c4c ends here

// [[file:../ipi.note::42437aac][42437aac]]
#[derive(Args, Debug)]
/// Compute molecule stream using any package (CP2K, SIESTA, etc) in i-PI
/// protocol
struct ProxyClient {
    /// The file containing molecule for computation
    mol_file: PathBuf,

    /// Path to lock file containing server address for connection
    #[clap(short = 'w', default_value = "gosh-ipi.lock")]
    lock_file: PathBuf,
}

impl ProxyClient {
    async fn enter_main(&self) -> Result<()> {
        let addr = gut::fs::read_file(&self.lock_file)?;
        let client = client::Client::connect(dbg!(addr.trim()));
        let mol = Molecule::from_file(&self.mol_file)?;
        let mp = client.compute_molecule(&mol).await?;
        dbg!(mp);

        Ok(())
    }
}
// 42437aac ends here

// [[file:../ipi.note::cf06c8c7][cf06c8c7]]
#[derive(Args, Debug)]
struct ProxyServer {
    /// Path to lock file for writing server address.
    #[clap(short = 'w', default_value = "gosh-ipi.lock")]
    lock_file: PathBuf,
}

impl ProxyServer {
    async fn enter_main(&self) -> Result<()> {
        rest::enter_main(&self.lock_file).await?;
        Ok(())
    }
}
// cf06c8c7 ends here

// [[file:../ipi.note::34481538][34481538]]
#[derive(Subcommand, Debug)]
enum ProxyCmd {
    /// Client side action for ipi-proxy
    Client(ProxyClient),
    /// Server side action for ipi-proxy
    Server(ProxyServer),
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
/// ipi-proxy for computation of a molecule
pub struct IpiProxyCli {
    #[clap(flatten)]
    verbose: Verbosity,

    /// The server mode to start.
    #[clap(subcommand)]
    cmd: ProxyCmd,
}

impl IpiProxyCli {
    #[tokio::main]
    pub async fn enter_main() -> Result<()> {
        let args = Self::from_args();
        args.verbose.setup_logger();

        match args.cmd {
            ProxyCmd::Client(client) => client.enter_main().await?,
            ProxyCmd::Server(server) => server.enter_main().await?,
        }

        Ok(())
    }
}
// 34481538 ends here
