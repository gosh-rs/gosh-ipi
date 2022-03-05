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
    #[tokio::main]
    /// Request remote server compute `mol` using external code in i-PI protocol
    pub async fn compute_molecule(&self, mol: &Molecule) -> Result<ModelProperties> {
        info!("Request server to compute molecule {}", mol.title());
        let x = self.post("mol", &mol).await?;
        let mol = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(mol)
    }
}
// 285a8db0 ends here

// [[file:../ipi.note::389c909a][389c909a]]
use socket::Socket;

/// Server side for proxying i-PI computation requests to external code
pub struct Server;

impl Server {
    /// Wait for incoming task and forward computation to external code in i-PI protocol
    async fn serve_incoming_task(mut task: TaskReceiver) {
        // FIXME: remove unwrap, and allow custom port
        let mut ipi_server = Socket::bind("localhost", 12345, false).await.unwrap();
        // if let Err(err) = task.compute_molecule_with(&mut ipi_server).await {
        if let Err(err) = ipi_server.serve_channel(&mut task).await {
            error!("{err:?}");
        }
    }

    #[tokio::main]
    /// Enter point for command line usage
    pub async fn enter_main(lock_file: &Path) -> Result<()> {
        let addr = socket::get_free_tcp_address().ok_or(format_err!("no free tcp addr"))?;
        println!("listening on {addr:?}");
        let _lock = LockFile::new(lock_file, addr);

        let (task_rx, task_tx) = Task::new().split();
        let h1 = tokio::spawn(async move { Self::run_restful(addr, task_tx).await });
        let h2 = tokio::spawn(async move { Self::serve_incoming_task(task_rx).await });
        tokio::try_join!(h1, h2)?;
        Ok(())
    }
}
// 389c909a ends here
