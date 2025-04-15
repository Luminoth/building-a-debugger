mod options;

use nix::sys::wait;
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

#[allow(dead_code)]
fn print_stop_reason(process: &sdb::Process, status: wait::WaitStatus) {
    match process.get_state() {
        sdb::ProcessState::Stopped => info!(
            "Process {} stopped with signal {:?}",
            process.get_id(),
            status
        ),
        sdb::ProcessState::Exited => info!(
            "Process {} exited with status {:?}",
            process.get_id(),
            status
        ),
        sdb::ProcessState::Terminated => {
            info!(
                "Process {} terminated with signal {:?}",
                process.get_id(),
                status
            )
        }
        _ => (),
    }
}

fn handle_command(process: &mut sdb::Process, command: impl Into<String>) -> anyhow::Result<()> {
    let command = command.into();
    let v = command.split_whitespace().collect::<Vec<_>>();
    if v.is_empty() {
        return Ok(());
    }

    let command = v[0];
    let _args = &v[1..];

    if command.starts_with("cont") {
        info!("Resuming process ...");
        process.resume()?;
        // TODO: this is hanging for some reason
        /*let status = process.wait_on_signal()?;
        print_stop_reason(process, status);*/
    }

    Ok(())
}

fn run(mut process: sdb::Process) -> anyhow::Result<()> {
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
                handle_command(&mut process, line)?;
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

fn main() -> anyhow::Result<()> {
    let options = argh::from_env::<Options>();

    init_logging()?;

    let process = match options.command {
        Command::Attach(command) => {
            info!("Attaching to process {} ...", command.process_id);
            sdb::Process::attach(command.process_id)?
            // TODO: if the error from this is operation not permitted
            // print something like gdb does about how
            // "if the uid is the same, fix this at the system level"
        }
        Command::Spawn(command) => {
            info!("Spawning process from {} ...", command.path);
            sdb::Process::launch(command.path, true)?
        }
    };

    run(process)
}
