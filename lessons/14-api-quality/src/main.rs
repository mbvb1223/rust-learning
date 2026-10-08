use api_quality::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, so that every later failure is logged.
    api_quality::init_tracing();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("invalid configuration: {err}");
            // `exit` skips destructors (lesson 05); harmless here, nothing has been opened yet.
            std::process::exit(1);
        }
    };

    let pool = api_quality::connect(&config.database_url).await?;
    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");

    api_quality::serve(
        listener,
        api_quality::app(pool.clone()),
        api_quality::shutdown_signal(),
    )
    .await?;

    // In-flight requests have finished; now release the database.
    pool.close().await;
    tracing::info!("shut down cleanly");
    Ok(())
}
