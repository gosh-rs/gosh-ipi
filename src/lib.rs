// [[file:../ipi.note::45bd773d][45bd773d]]
use gosh_core::*;
use gut::prelude::*;

use gchemol::prelude::*;
use gchemol::{Atom, Lattice, Molecule};
// 45bd773d ends here

// [[file:../ipi.note::2783ec3a][2783ec3a]]
mod codec;
mod ipi;

#[cfg(feature = "adhoc")]
pub mod docs {
    pub use super::codec::*;
}
// 2783ec3a ends here

// [[file:../ipi.note::04b72e76][04b72e76]]
/// The Message type sent from client side (the computation engine)
#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    /// The client code needs initializing data.
    NeedInit,
    /// The client code is ready to calculate the forces.
    Ready,
    /// The client has finished computing the potential and forces.
    HaveData,
}

/// The message sent from server side (application)
#[derive(Debug, Clone)]
pub enum ServerMessage {
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

/// The message sent by client code (VASP ...)
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
