use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Attach(AttachCommand),
}

/// Attach to a process
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "attach")]
pub struct AttachCommand {
    #[argh(positional)]
    pub process_id: i32,
}

/// Command line Options
#[derive(Debug, FromArgs)]
pub struct Options {
    #[argh(subcommand)]
    pub command: Command,
}
