// [[file:../ipi.note::475dbc7d][475dbc7d]]
use super::*;

use tokio::sync::{mpsc, oneshot};
// 475dbc7d ends here

// [[file:../ipi.note::214790a9][214790a9]]
type RxComputed = oneshot::Receiver<Computed>;
type TxComputed = oneshot::Sender<Computed>;
type IpiJob = (Molecule, TxComputed);
type TxInput = mpsc::Sender<IpiJob>;

pub type RxInput = mpsc::Receiver<IpiJob>;
// 214790a9 ends here

// [[file:../ipi.note::b55affa9][b55affa9]]
#[derive(Debug, Clone, Default)]
pub struct TaskSender {
    tx_inp: Option<TxInput>,
}

impl TaskSender {
    pub async fn request_compute_molecule(&self, mol: Molecule) -> Result<Computed> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_inp.as_ref().unwrap().send((mol, tx)).await?;
        let computed = rx.await?;
        Ok(computed)
    }
}
// b55affa9 ends here

// [[file:../ipi.note::f45eafe9][f45eafe9]]
#[derive(Debug)]
pub struct TaskReceiver {
    rx_inp: RxInput,
}

impl TaskReceiver {
    /// Receives the next task for this receiver.
    pub async fn recv(&mut self) -> Option<IpiJob> {
        self.rx_inp.recv().await
    }
}

fn new_interactive_task() -> (TaskReceiver, TaskSender) {
    let (tx_inp, rx_inp) = tokio::sync::mpsc::channel(1);

    let server = TaskReceiver { rx_inp };
    let client = TaskSender { tx_inp: tx_inp.into() };

    (server, client)
}
// f45eafe9 ends here

// [[file:../ipi.note::3f19ae12][3f19ae12]]
pub struct Task {
    sender: TaskSender,
    receiver: TaskReceiver,
}

impl Task {
    /// Create a task channel for computation of molecule in client/server
    /// architecture
    pub fn new() -> Self {
        let (receiver, sender) = new_interactive_task();
        Self { sender, receiver }
    }

    /// Splits a single task into separate read and write half
    pub fn split(self) -> (TaskReceiver, TaskSender) {
        match self {
            Self {
                sender: tx,
                receiver: rx,
            } => (rx, tx),
        }
    }
}
// 3f19ae12 ends here
