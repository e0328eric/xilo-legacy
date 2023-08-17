mod command;
mod config;
mod error;
mod initializer;
mod remover;
mod space_shower;
mod terminal;

use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use initializer::Initializer;
use remover::FileTypeToRemove;

macro_rules! unwrap_xilo_err {
    ($to_unwrap: expr) => {
        match $to_unwrap {
            Ok(val) => val,
            Err(err) => {
                terminal::print(
                    terminal::MessageType::Error,
                    format!("{}\n", err.to_string()),
                )
                .unwrap();
                return ExitCode::FAILURE;
            }
        }
    };
}

fn main() -> ExitCode {
    let command::XiloCommand {
        filenames,
        recursive,
        force,
        permanent,
        show_space,
        raw,
    } = command::XiloCommand::parse();

    if !permanent && !show_space && filenames.is_empty() {
        command::XiloCommand::command().print_long_help().unwrap();
        return ExitCode::SUCCESS;
    }

    let config = unwrap_xilo_err!(config::XiloConfig::new());

    let initializer = unwrap_xilo_err!(Initializer::new(
        config,
        if filenames.is_empty() {
            permanent
        } else {
            false
        },
    ))
    .recursive(recursive)
    .force(force)
    .permanent(permanent)
    .show_space(show_space);

    if !filenames.is_empty() && recursive && !filenames.iter().any(|path| path.is_dir()) {
        terminal::print(
            terminal::MessageType::Warning,
            "Recursive flag effects nothing while removing a file.\n".to_string(),
        )
        .unwrap();
    }

    if show_space {
        let space_shower = initializer.make_space_shower();
        let raw_trashbin_space = unwrap_xilo_err!(space_shower.get_raw_space());
        let trashbin_space = unwrap_xilo_err!(space_shower.get_space());
        terminal::print(
            terminal::MessageType::Note,
            format!(
                "The space of the current trashbin is {display_space}\n",
                display_space = if raw {
                    format!("{raw_trashbin_space}B")
                } else {
                    trashbin_space
                },
            ),
        )
        .unwrap();
    }

    let mut handler;
    for name in filenames {
        if name.is_dir() && !name.is_symlink() {
            let name = unwrap_xilo_err!(name.canonicalize().map_err(error::XiloError::IOErr));
            handler = initializer.make_remover(FileTypeToRemove::Directory, name);
        } else {
            handler = initializer.make_remover(FileTypeToRemove::File, name);
        }
        unwrap_xilo_err!(handler.execute());
    }

    ExitCode::SUCCESS
}
