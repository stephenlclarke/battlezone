//! Command-line entry point; informational commands never initialize the window.
use anyhow::{Result, bail};

const HELP: &str = "Battlezone launches the game in a wgpu window.

Usage: battlezone [--help | --version]

Controls:
  Enter / 1       Start
  Q / A           Left tread forward / reverse
  P / L           Right tread forward / reverse
  Arrow keys      Drive / turn
  Space           Fire
  H               Toggle autopilot
  Escape          Quit

Interactive play requires a graphical desktop and a wgpu-compatible GPU.";

fn main() {
    std::process::exit(exit_code_for_result(run()));
}

fn run() -> Result<()> {
    match classify_args(std::env::args().skip(1))? {
        Command::Play => battlezone::app::run(),
        Command::Help => {
            println!("{HELP}");
            Ok(())
        }
        Command::Version => {
            println!("battlezone {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Play,
    Help,
    Version,
}

fn classify_args(args: impl IntoIterator<Item = String>) -> Result<Command> {
    let mut args = args.into_iter();
    let Some(arg) = args.next() else {
        return Ok(Command::Play);
    };
    if args.next().is_some() {
        bail!("expected at most one option; use --help");
    }
    match arg.as_str() {
        "--help" | "-h" => Ok(Command::Help),
        "--version" | "-V" => Ok(Command::Version),
        _ => bail!("unknown option {arg:?}; use --help"),
    }
}

fn exit_code_for_result(result: Result<()>) -> i32 {
    if let Err(err) = result {
        eprintln!("battlezone: {err:#}");
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_is_zero_for_success() {
        assert_eq!(exit_code_for_result(Ok(())), 0);
    }

    #[test]
    fn exit_code_is_one_for_failure() {
        assert_eq!(exit_code_for_result(Err(anyhow::anyhow!("boom"))), 1);
    }

    #[test]
    fn informational_commands_and_default_play_are_classified() {
        assert_eq!(classify_args(Vec::new()).unwrap(), Command::Play);
        for (arg, command) in [
            ("--help", Command::Help),
            ("-h", Command::Help),
            ("--version", Command::Version),
            ("-V", Command::Version),
        ] {
            assert_eq!(classify_args([arg.to_string()]).unwrap(), command);
        }
    }

    #[test]
    fn unknown_and_extra_options_are_rejected() {
        assert!(classify_args(["--unknown".to_string()]).is_err());
        assert!(classify_args(["--help".to_string(), "--version".to_string()]).is_err());
    }
}
