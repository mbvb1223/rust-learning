//! `serve` over a real socket. The shutdown signal is a channel instead of Ctrl+C, so the test
//! decides when it fires.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, oneshot};
use tokio::task::JoinHandle;

/// A minimal HTTP/1.1 client: returns the raw response, headers and body.
async fn http_get(addr: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

type Server = JoinHandle<std::io::Result<()>>;

const LIMIT: Duration = Duration::from_secs(5);

async fn start(router: Router) -> (SocketAddr, oneshot::Sender<()>, Server) {
    // Port 0: the OS picks a free port, so parallel tests never collide.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(api_quality::serve(listener, router, async {
        shutdown_rx.await.ok();
    }));
    (addr, shutdown_tx, server)
}

async fn assert_stops(server: Server) {
    let result = tokio::time::timeout(LIMIT, server)
        .await
        .expect("serve kept running after the shutdown signal")
        .expect("serve panicked");
    assert!(result.is_ok(), "serve returned {result:?}");
}

#[tokio::test]
async fn serve_runs_until_the_shutdown_signal() {
    let router = Router::new().route("/", get(|| async { "hello" }));
    let (addr, shutdown, server) = start(router).await;

    let response = http_get(addr, "/").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("hello"), "{response}");
    assert!(!server.is_finished());

    shutdown.send(()).unwrap();
    assert_stops(server).await;
}

#[tokio::test]
async fn shutdown_lets_in_flight_requests_finish() {
    let started = Arc::new(Notify::new());
    let handler_started = started.clone();
    let router = Router::new().route(
        "/slow",
        get(move || async move {
            handler_started.notify_one();
            tokio::time::sleep(Duration::from_millis(300)).await;
            "finished"
        }),
    );
    let (addr, shutdown, server) = start(router).await;

    let client = tokio::spawn(http_get(addr, "/slow"));
    tokio::time::timeout(LIMIT, started.notified())
        .await
        .expect("the request never reached the handler");
    shutdown.send(()).unwrap();

    let response = client.await.unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("finished"), "{response}");
    assert_stops(server).await;
}
