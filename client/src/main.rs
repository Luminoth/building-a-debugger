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
        Command::Attach(command) => {
            info!("Attaching to process {} ...", command.process_id);
            sdb::attach(command.process_id)?;
            // TODO: if the error from this is operation not permitted
            // print something like gdb does about how
            // "if the uid is the same, fix this at the system level"
        }
        Command::Spawn(command) => {
            info!("Spawning process from {} ...", command.path);
            sdb::spawn_and_attach(command.path)?;
        }
    }

    Ok(())
}
