use axum_lesson::{AppState, app};

#[tokio::main]
async fn main() {
    // 0.0.0.0, not 127.0.0.1: inside Docker, loopback isn't reachable through a published port.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port 3000 is free");
    println!("listening on http://localhost:3000 (Ctrl+C to stop)");

    axum::serve(listener, app(AppState::default()))
        // In Docker the server runs as PID 1, which ignores Ctrl+C unless the program handles it.
        // Lesson 14 covers graceful shutdown properly.
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Ctrl+C handler installs");
        })
        .await
        .expect("server runs until stopped");
}
