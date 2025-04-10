mod options;

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use options::*;

fn init_logging() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let options = argh::from_env::<Options>();

    init_logging()?;

    match options.command {
        Command::Attach(command) => info!("attach to {}", command.process_id),
    }

    Ok(())
}
