mod options;

use rustyline::{
    DefaultEditor,
    error::ReadlineError,
    history::{History, SearchDirection},
};
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

fn handle_command(command: impl Into<String>) {
    let command = command.into().trim().to_owned();
    if command.is_empty() {
        return;
    }

    info!("command: {}", command);
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

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(mut line) => {
                if line.trim().is_empty() {
                    let history = rl.history();
                    if history.len() > 0 {
                        line = history
                            .get(history.len() - 1, SearchDirection::Forward)?
                            .unwrap()
                            .entry
                            .into();
                    }
                } else {
                    rl.add_history_entry(line.as_str())?;
                }
                handle_command(line);
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                Err(err)?;
            }
        }
    }

    Ok(())
}
