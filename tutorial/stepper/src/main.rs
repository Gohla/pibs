use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

mod app;
mod modification;
mod output;
mod cargo;
mod stepper;
mod util;

#[derive(Debug, Parser)]
#[command()]
struct Args {
  /// Print info trace messages to stdout
  #[arg(short, long)]
  info: bool,
  /// Print debug trace messages to stdout.
  #[arg(short, long)]
  debug: bool,

  /// Command to run. Defaults to step-all.
  #[command(subcommand)]
  command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
  /// Go through all steps to verify them and generate outputs, only stopping if a step fails.
  StepAll {
    /// Destination root directory where all source files are created and modified during stepping. Defaults to a temporary directory.
    #[arg(short, long)]
    destination_root_directory: Option<PathBuf>,
    /// Whether to skip cargo commands for steps, effectively disabling verification.
    #[arg(long)]
    skip_cargo: bool,
    /// Whether to skip creating outputs.
    #[arg(long)]
    skip_outputs: bool,
  }
}

impl Default for Command {
  fn default() -> Self {
    Command::StepAll {
      destination_root_directory: None,
      skip_cargo: false,
      skip_outputs: false,
    }
  }
}

fn main() {
  dotenv::dotenv().ok();

  let args = Args::parse();

  let mut level = None;
  if args.info {
    level = Some(LevelFilter::INFO.into());
  }
  if args.debug {
    level = Some(LevelFilter::DEBUG.into());
  }
  let filter = if let Some(level) = level {
    EnvFilter::builder()
      .with_default_directive(level)
      .parse_lossy("")
  } else {
    EnvFilter::from_env("MAIN_LOG")
  };

  let format = fmt::format()
    .with_level(false)
    .with_target(false)
    .with_thread_ids(false)
    .with_thread_names(false)
    .without_time()
    .compact();
  tracing_subscriber::registry()
    .with(
      fmt::layer()
        .event_format(format)
        .with_writer(std::io::stdout)
        .with_filter(filter)
    )
    .init();

  let command = args.command.unwrap_or_default();
  match command {
    Command::StepAll { destination_root_directory, skip_cargo, skip_outputs } => {
      let destination_root_directory = destination_root_directory.unwrap_or_else(|| tempfile::tempdir().expect("failed to create temporary directory").into_path());
      app::step_all(destination_root_directory, !skip_cargo, !skip_outputs);
    }
  }
}
