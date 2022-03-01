// [[file:../ipi.note::609c7b71][609c7b71]]
use super::*;
use socket::*;
// 609c7b71 ends here

// [[file:../ipi.note::*client][client:1]]

// client:1 ends here

// [[file:../ipi.note::c57d968b][c57d968b]]
#[derive(Debug)]
pub struct IpiProxy {
    ipi_server: IpiListener,
    task_server: TaskReceiver,
}
// c57d968b ends here

// [[file:../ipi.note::5537d196][5537d196]]
impl IpiProxy {
    pub async fn new() -> Result<Self> {
        let (task_server, task_client) = new_interactive_task();
        let ipi_server = Socket::bind("localhost", 12345, false).await?;

        let s = Self {
            task_server,
            ipi_server,
        };
        Ok(s)
    }

    async fn start(&mut self) -> Result<()> {
        self.task_server.compute_molecule_with(&mut self.ipi_server).await?;
        Ok(())
    }
}
// 5537d196 ends here
