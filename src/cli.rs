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

// [[file:../ipi.note::44b57b90][44b57b90]]
#[tokio::main]
pub async fn enter_main() -> Result<()> {
    use gchemol::prelude::*;
    use gchemol::Molecule;
    use gosh_model::BlackBoxModel;

    let args = IpiCli::from_args();
    args.verbose.setup_logger();

    let mut bbm = BlackBoxModel::from_dir(&args.bbm)?;
    let mol = Molecule::from_file(&args.mol)?;

    // default port as in ipi
    let sock = Socket::connect(&args.sock, 12345, true).await?;
    ipi::ipi_client(bbm, mol, sock).await?;
    dbg!();

    Ok(())
}
// 44b57b90 ends here
