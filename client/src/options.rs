use argh::FromArgs;

#[derive(Debug, PartialEq, FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Attach(AttachCommand),
    Spawn(SpawnCommand),
}

/// Attach to a process
#[derive(Debug, PartialEq, FromArgs)]
#[argh(subcommand, name = "attach")]
pub struct AttachCommand {
    #[argh(positional)]
    pub process_id: i32,
}

/// Spawn a process and attach to it
#[derive(Debug, PartialEq, FromArgs)]
#[argh(subcommand, name = "spawn")]
pub struct SpawnCommand {
    #[argh(positional)]
    pub path: String,
}

/// Command line Options
#[derive(Debug, FromArgs)]
pub struct Options {
    #[argh(subcommand)]
    pub command: Command,
}
