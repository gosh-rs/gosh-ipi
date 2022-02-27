// [[file:../ipi.note::3d2c01c2][3d2c01c2]]
use super::*;

use gosh_model::ModelProperties;
// 3d2c01c2 ends here

// [[file:../ipi.note::aa8d1d68][aa8d1d68]]
mod client;
mod server;
// aa8d1d68 ends here

// [[file:../ipi.note::285a8db0][285a8db0]]
pub use client::Client;

impl Client {
    pub async fn compute_molecule(&self, mol: &Molecule) -> Result<ModelProperties> {
        info!("computing molecule {}", mol.title());
        let x = self.post("mol", &mol).await?;
        let mol = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(mol)
    }
}
// 285a8db0 ends here

// [[file:../ipi.note::389c909a][389c909a]]
pub use server::Server;

impl Server {
    // high level details for computation of molecule
    //
    // this function will be called by sub mod server
    fn compute_mol_using_ipi(mol: Molecule) -> ModelProperties {
        // FIXME: using i-PI
        ModelProperties::default()
    }
}
// 389c909a ends here
