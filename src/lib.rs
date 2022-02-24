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

// mod proxy;
// 2783ec3a ends here

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
