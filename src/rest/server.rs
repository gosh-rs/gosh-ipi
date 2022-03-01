// [[file:../../ipi.note::3d2c01c2][3d2c01c2]]
use super::*;

use axum::Json;
use std::net::SocketAddr;
// 3d2c01c2 ends here

// [[file:../../ipi.note::ad35d99c][ad35d99c]]
// type State = std::sync::Arc<TaskSender>;
type State = TaskSender;
// ad35d99c ends here

// [[file:../../ipi.note::7157f9ad][7157f9ad]]
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;

async fn compute_mol(Json(mol): Json<Molecule>, client: Extension<State>) -> impl IntoResponse {
    match client.request_compute_molecule(mol).await {
        Ok(computed) => {
            // let mp = Server::compute_mol_using_ipi(mol);
            // (StatusCode::OK, Json(mp))
            todo!();
        }
        Err(err) => {
            dbg!(err);
            todo!();
        }
    }
}
// 7157f9ad ends here

// [[file:../../ipi.note::59c3364a][59c3364a]]
macro_rules! build_app_with_routes {
    () => {{
        use axum::routing::post;
        use axum::AddExtensionLayer;

        let state = State::default();
        axum::Router::new()
            .route("/mol", post(compute_mol))
            .layer(AddExtensionLayer::new(state))
    }};
}
// 59c3364a ends here

// [[file:../../ipi.note::415dc72b][415dc72b]]
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
// 415dc72b ends here

// [[file:../../ipi.note::f4a1566d][f4a1566d]]
impl Server {
    pub async fn enter_main(lock_file: &Path) -> Result<()> {
        let app = build_app_with_routes!();

        // run it
        let addr = socket::get_free_tcp_address().ok_or(format_err!("no free tcp addr"))?;
        println!("listening on {addr:?}");
        let _lock = LockFile::new(lock_file, addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}
// f4a1566d ends here
