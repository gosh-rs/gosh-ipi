// [[file:../ipi.note::609c7b71][609c7b71]]
use super::*;
use socket::*;
// 609c7b71 ends here

// [[file:../ipi.note::*client][client:1]]

// client:1 ends here

// [[file:../ipi.note::*server][server:1]]

// server:1 ends here

// [[file:../ipi.note::5537d196][5537d196]]
#[derive(Debug)]
struct IpiProxy {
    server: IpiListener,
    rx_inp: RxInput,
    rx_out: RxOutput,
}

impl IpiProxy {
    async fn start(self) -> Result<()> {
        ipi::ipi_server(self.server, self.rx_inp).await?;

        Ok(())
    }
}
// 5537d196 ends here
