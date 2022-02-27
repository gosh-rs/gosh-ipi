// [[file:../ipi.note::45bd773d][45bd773d]]
use gosh_core::*;
use gut::prelude::*;

use gchemol::prelude::*;
use gchemol::{Atom, Lattice, Molecule};
// 45bd773d ends here

// [[file:../ipi.note::2783ec3a][2783ec3a]]
mod codec;
mod ipi;
mod socket;

mod proxy;
mod rest;
// 2783ec3a ends here

// [[file:../ipi.note::f9b302af][f9b302af]]
// input type
type RxComputed = tokio::sync::oneshot::Receiver<Computed>;
type TxComputed = tokio::sync::oneshot::Sender<Computed>;
type RxInput = tokio::sync::mpsc::Receiver<(Molecule, TxComputed)>;
type TxInput = tokio::sync::mpsc::Sender<Molecule>;

// output type
type RxOutput = tokio::sync::mpsc::Receiver<String>;
type TxOutput = tokio::sync::mpsc::Sender<String>;
// f9b302af ends here

// [[file:../ipi.note::929936e0][929936e0]]
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LockFile {
    file: std::fs::File,
    path: PathBuf,
}

impl LockFile {
    fn create(path: &Path) -> Result<LockFile> {
        use fs2::*;

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .context("Could not create ID file")?;

        // https://docs.rs/fs2/0.4.3/fs2/trait.FileExt.html
        file.try_lock_exclusive()
            .context("Could not lock ID file; Is the daemon already running?")?;

        Ok(LockFile {
            file,
            path: path.to_owned(),
        })
    }

    fn write_msg(&mut self, msg: impl std::fmt::Display) -> Result<()> {
        writeln!(&mut self.file, "{msg}").context("Could not write ID file")?;
        self.file.flush().context("Could not flush ID file")
    }

    /// Create a lockfile in `path` containing `msg`
    pub fn new(path: &Path, msg: impl std::fmt::Display) -> Result<Self> {
        let mut lockfile = Self::create(path)?;
        lockfile.write_msg(msg)?;
        Ok(lockfile)
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
// 929936e0 ends here

// [[file:../ipi.note::04b72e76][04b72e76]]
/// The status of the client
#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    /// The client is ready to receive forcefield parameters.
    NeedInit,
    /// The client is ready to receive position and cell data.
    Ready,
    /// The client is ready to send force data
    HaveData,
    /// The client is running
    Up,
    /// The client has disconnected
    Disconnected,
    /// The connection has timed out
    TimeOut,
}

/// The message sent from the driver code (i-PI ...)
#[derive(Debug, Clone)]
pub enum DriverMessage {
    /// Request the status of the client code
    Status,

    /// Send the client code the initialization data followed by an integer
    /// corresponding to the bead index, another integer giving the number of
    /// bits in the initialization string, and finally the initialization string
    /// itself.
    Init(InitData),

    /// Send the client code the cell and cartesion positions.
    PosData(Molecule),

    /// Get the potential and forces computed by client code
    GetForce,

    /// Request to exit
    Exit,
}

/// The message sent from client code (CP2K, SIESTA, VASP ...)
#[derive(Debug, Clone)]
pub enum ClientMessage {
    ForceReady(Computed),
    Status(ClientStatus),
}

#[derive(Debug, Clone)]
pub struct InitData {
    ibead: usize,
    nbytes: usize,
    init: String,
}

impl InitData {
    fn new(ibead: usize, init: &str) -> Self {
        Self {
            ibead,
            nbytes: init.len(),
            init: init.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Computed {
    energy: f64,
    forces: Vec<[f64; 3]>,
    virial: [f64; 9],
    extra: String,
}
// 04b72e76 ends here

// [[file:../ipi.note::242ad86a][242ad86a]]
#[cfg(feature = "adhoc")]
/// Docs for local mods
pub mod docs {
    macro_rules! export_doc {
        ($l:ident) => {
            pub mod $l {
                pub use crate::$l::*;
            }
        };
    }

    export_doc!(codec);
    export_doc!(socket);
    export_doc!(ipi);
}
// 242ad86a ends here

// [[file:../ipi.note::45e64f3f][45e64f3f]]
pub mod cli;
// 45e64f3f ends here
