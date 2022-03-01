// [[file:../ipi.note::3d2c01c2][3d2c01c2]]
use super::*;

use socket::IpiListener;
use task::{Task, TaskReceiver, TaskSender};

use gosh_model::ModelProperties;
// 3d2c01c2 ends here

// [[file:../ipi.note::aa8d1d68][aa8d1d68]]
mod client;
mod server;
// aa8d1d68 ends here

// [[file:../ipi.note::285a8db0][285a8db0]]
pub use client::Client;

impl Client {
    /// Request remote server compute `mol` using external code in i-PI protocol
    pub async fn compute_molecule(&self, mol: &Molecule) -> Result<ModelProperties> {
        info!("computing molecule {}", mol.title());
        let x = self.post("mol", &mol).await?;
        let mol = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(mol)
    }
}
// 285a8db0 ends here

// [[file:../ipi.note::389c909a][389c909a]]
use socket::Socket;

/// Server side for proxying i-PI computation requests to external code
pub struct Server {
    ipi_server: IpiListener,
    task: Task,
}

impl Server {
    pub async fn new() -> Result<Self> {
        // FIXME: using clap arguments
        let ipi_server = Socket::bind("localhost", 12345, false).await?;

        let s = Self {
            task: Task::new(),
            ipi_server,
        };
        Ok(s)
    }
}
// 389c909a ends here
