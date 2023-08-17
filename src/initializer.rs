use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::config::XiloConfig;
use crate::error::{self, XiloError};
use crate::remover::{FileTypeToRemove, Remover};
use crate::space_shower::SpaceShower;
use crate::terminal::{self, MessageType};

pub struct Initializer {
    trashbin_path: PathBuf,
    recursive: bool,
    force: bool,
    permanent: bool,
    show_space: bool,
}

impl Initializer {
    pub fn new(config: Option<XiloConfig>, reset_trashbin: bool) -> error::Result<Self> {
        let default_trashbin_path = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|mut h| {
                    h.push(".Trash");
                    h
                })
                .ok_or(XiloError::CannotFindTrashbinPath)?
        } else {
            let mut tmp = dirs::cache_dir().ok_or(XiloError::CannotFindCacheDirPath)?;
            tmp.push("xilo");
            tmp
        };

        let trashbin_path = if let Some(config) = config {
            if let Some(path) = config.trashbin_path {
                if cfg!(unix) {
                    expand_tilde(path).ok_or(XiloError::CannotFindTrashbinPath)?
                } else {
                    path
                }
            } else {
                default_trashbin_path
            }
        } else {
            default_trashbin_path
        };

        if reset_trashbin {
            terminal::print(
                MessageType::Warning,
                "Are you sure to empty trashbin? (y/N): ".to_string(),
            )?;
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;

            match buf.trim() {
                "y" | "Y" | "yes" | "Yes" | "YES" => {
                    let dir_iter = fs::read_dir(&trashbin_path)?;
                    for entry in dir_iter {
                        let path = entry?.path();
                        if path.is_dir() {
                            fs::remove_dir_all(path).map_err(XiloError::RippingTrashbinFailed)?;
                        } else {
                            fs::remove_file(path).map_err(XiloError::RippingTrashbinFailed)?;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Err(io::ErrorKind::NotFound) = fs::read_dir(&trashbin_path).map_err(|err| err.kind())
        {
            match fs::create_dir(&trashbin_path) {
                Ok(()) => {}
                Err(err) => {
                    return Err(XiloError::XiloInitFailed(err).into());
                }
            }
        }

        Ok(Self {
            trashbin_path,
            recursive: false,
            force: false,
            permanent: false,
            show_space: false,
        })
    }

    #[inline]
    pub fn recursive(mut self, val: bool) -> Self {
        self.recursive = val;
        self
    }

    #[inline]
    pub fn force(mut self, val: bool) -> Self {
        self.force = val;
        self
    }

    #[inline]
    pub fn permanent(mut self, val: bool) -> Self {
        self.permanent = val;
        self
    }

    #[inline]
    pub fn show_space(mut self, val: bool) -> Self {
        self.show_space = val;
        self
    }

    pub fn make_remover(&self, handle_type: FileTypeToRemove, name: PathBuf) -> Remover<'_> {
        Remover::new(
            handle_type,
            name,
            &self.trashbin_path,
            self.recursive,
            self.force,
            self.permanent,
        )
    }

    pub fn make_space_shower(&self) -> SpaceShower<'_> {
        SpaceShower::new(&self.trashbin_path)
    }
}

// NOTE: Code stolen from https://stackoverflow.com/questions/54267608/expand-tilde-in-rust-path-idiomatically
#[cfg(unix)]
fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}
