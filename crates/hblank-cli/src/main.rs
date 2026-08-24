use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use hblank_cli::{
    CatalogOptions, DevOptions, InitOptions, TestOptions, initialize, run_dev, run_list, run_tests,
};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Develop GPUI components in isolation",
    after_long_help = "OPEN A FIXTURE DIRECTLY:
  hblank dev --fixture src/button.hblank.rs

Run 'hblank dev --help' for relative-path rules and complete usage details."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add Hblank configuration and preview boilerplate to a GPUI package.
    Init {
        /// Rust package root to initialize.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Use a local Hblank runtime instead of the published crate.
        #[arg(long, value_name = "PATH")]
        runtime_path: Option<PathBuf>,
    },
    /// Launch the GPUI harness, optionally opening one fixture source file directly.
    #[command(
        long_about = "Launch the GPUI component development harness and watch for changes. Pass --fixture to select a fixture by its source-file path on the initial launch.",
        after_long_help = "PATH RULES:
  Relative fixture paths are resolved from --project. Absolute paths are accepted.
  The file must match the fixture file patterns in .hblank/config.toml.
  If a file registers multiple fixtures, the first in navigation order opens.

USAGE:
  hblank dev --fixture src/button.hblank.rs
  hblank dev --project crates/ui --fixture src/card.hblank.rs
  hblank dev --fixture /absolute/path/to/src/badge.hblank.rs"
    )]
    Dev {
        /// Initialized Rust package root to develop.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Open directly to the first registered fixture in this fixture source file.
        ///
        /// Relative paths are resolved from --project; absolute paths are also accepted.
        #[arg(long, value_name = "PATH")]
        fixture: Option<PathBuf>,
        /// Open one exact registered fixture by canonical project-relative path#function id.
        #[arg(long, value_name = "ID", conflicts_with = "fixture")]
        fixture_id: Option<String>,
    },
    /// Build and print registered components and canonical fixture ids.
    List {
        /// Initialized Rust package root whose catalog should be listed.
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Run explicit inline Rust tests from the generated Hblank preview target.
    Test {
        /// Initialized Rust package root whose fixture tests should run.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Optional Cargo test-name filter.
        #[arg(long, value_name = "FILTER")]
        filter: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init {
            project,
            runtime_path,
        } => {
            let mut options = InitOptions::new(project);
            options.runtime_path = runtime_path;
            let report = initialize(&options)?;
            println!("Initialized Hblank in {}", report.project_root.display());
            for path in report.created {
                println!("  create {}", path.display());
            }
        }
        Command::Dev {
            project,
            fixture,
            fixture_id,
        } => {
            let mut options = DevOptions::new(project);
            options.fixture = fixture;
            options.fixture_id = fixture_id;
            run_dev(&options)?;
        }
        Command::List { project } => run_list(&CatalogOptions::new(project))?,
        Command::Test { project, filter } => {
            let mut options = TestOptions::new(project);
            options.filter = filter;
            run_tests(&options)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn top_level_help_points_to_direct_fixture_launch() {
        let mut command = Cli::command();
        let mut output = Vec::new();
        command
            .write_long_help(&mut output)
            .expect("top-level help should render");
        let help = String::from_utf8(output).expect("help should be UTF-8");

        assert!(help.contains("hblank dev --fixture src/button.hblank.rs"));
        assert!(help.contains("Run 'hblank dev --help'"));
    }

    #[test]
    fn list_help_documents_registered_catalog() {
        let mut command = Cli::command();
        let list = command
            .find_subcommand_mut("list")
            .expect("list subcommand should exist");
        let mut output = Vec::new();
        list.write_long_help(&mut output)
            .expect("list help should render");
        let help = String::from_utf8(output).expect("help should be UTF-8");

        assert!(help.contains("canonical fixture ids"));
    }

    #[test]
    fn test_help_documents_generated_cargo_target() {
        let mut command = Cli::command();
        let test = command
            .find_subcommand_mut("test")
            .expect("test subcommand should exist");
        let mut output = Vec::new();
        test.write_long_help(&mut output)
            .expect("test help should render");
        let help = String::from_utf8(output).expect("help should be UTF-8");

        assert!(help.contains("explicit inline Rust tests"));
        assert!(help.contains("--filter <FILTER>"));
    }

    #[test]
    fn dev_help_documents_fixture_path_semantics() {
        let mut command = Cli::command();
        let dev = command
            .find_subcommand_mut("dev")
            .expect("dev subcommand should exist");
        let mut output = Vec::new();
        dev.write_long_help(&mut output)
            .expect("dev help should render");
        let help = String::from_utf8(output).expect("help should be UTF-8");

        assert!(help.contains("--fixture <PATH>"));
        assert!(help.contains("--fixture-id <ID>"));
        assert!(help.contains("path#function"));
        assert!(help.contains("Relative fixture paths are resolved from --project"));
        assert!(help.contains("If a file registers multiple fixtures"));
        assert!(help.contains("hblank dev --project crates/ui --fixture src/card.hblank.rs"));
    }
}
