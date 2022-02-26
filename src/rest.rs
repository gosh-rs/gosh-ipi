// [[file:../ipi.note::3d2c01c2][3d2c01c2]]
use super::*;

use axum::Json;
use std::net::SocketAddr;
// 3d2c01c2 ends here

// [[file:../ipi.note::7157f9ad][7157f9ad]]
use axum::http::StatusCode;
use axum::response::IntoResponse;

use gosh_model::ModelProperties;

async fn compute_mol(Json(mol): Json<Molecule>) -> impl IntoResponse {
    // FIXME: using i-PI protocol
    let mp = ModelProperties::default();
    (StatusCode::OK, Json(mp))
}
// 7157f9ad ends here

// [[file:../ipi.note::59c3364a][59c3364a]]
macro_rules! build_app_with_routes {
    () => {{
        use axum::routing::post;

        axum::Router::new().route("/mol", post(compute_mol))
    }};
}
// 59c3364a ends here

// [[file:../ipi.note::f4a1566d][f4a1566d]]
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
// f4a1566d ends here
