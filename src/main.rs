mod command;
mod config;
mod error;
mod initializer;
mod remover;
mod space_shower;

use std::io;

use clap::{CommandFactory, Parser};
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};

use initializer::Initializer;
use remover::FileTypeToRemove;

fn main() -> error::Result<()> {
    let command::XiloCommand {
        filenames,
        recursive,
        force,
        permanent,
        show_space,
        raw,
    } = command::XiloCommand::parse();

    if !permanent && !show_space && filenames.is_empty() {
        command::XiloCommand::command().print_long_help()?;
        return Ok(());
    }

    let config = config::XiloConfig::new()?;

    let initializer = Initializer::new(
        config,
        if filenames.is_empty() {
            permanent
        } else {
            false
        },
    )?
    .recursive(recursive)
    .force(force)
    .permanent(permanent)
    .show_space(show_space);

    if !filenames.is_empty() && recursive && !filenames.iter().any(|path| path.is_dir()) {
        execute!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta),
            Print("Note".to_string()),
            SetForegroundColor(Color::White),
            Print(": ".to_string()),
            Print("Recursive flag effects nothing while removing a file.\n".to_string()),
            ResetColor,
        )?;
    }

    if show_space {
        let space_shower = initializer.make_space_shower();
        let raw_trashbin_space = space_shower.get_raw_space()?;
        let trashbin_space = space_shower.get_space()?;
        execute!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta),
            Print("Note".to_string()),
            SetForegroundColor(Color::White),
            Print(": ".to_string()),
            Print(format!(
                "The space of the current trashbin is {display_space}\n",
                display_space = if raw {
                    format!("{raw_trashbin_space}B")
                } else {
                    trashbin_space
                },
            )),
            ResetColor,
        )?;
    }

    let mut handler;
    for name in filenames {
        if name.is_dir() && !name.is_symlink() {
            let name = name.canonicalize()?;
            handler = initializer.make_remover(FileTypeToRemove::Directory, name);
        } else {
            handler = initializer.make_remover(FileTypeToRemove::File, name);
        }
        handler.execute()?;
    }

    Ok(())
}
