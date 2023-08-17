use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64ct::{Base64Url, Encoding};
use sha2::{Digest, Sha256};

use crate::error::{self, XiloError};
use crate::terminal::{self, MessageType};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileTypeToRemove {
    File,
    Directory,
}

pub struct Remover<'p> {
    handle_type: FileTypeToRemove,
    name: PathBuf,
    trashbin_path: &'p Path,
    recursive: bool,
    force: bool,
    permanent: bool,
}

impl<'p> Remover<'p> {
    pub fn new(
        handle_type: FileTypeToRemove,
        name: PathBuf,
        trashbin_path: &'p Path,
        recursive: bool,
        force: bool,
        permanent: bool,
    ) -> Self {
        Self {
            handle_type,
            name,
            trashbin_path,
            recursive,
            force,
            permanent,
        }
    }

    pub fn execute(&self) -> error::Result<()> {
        use FileTypeToRemove::*;

        if self.handle_type == Directory && !self.recursive {
            return Err(XiloError::RemoveDirWithoutRecursiveFlag.into());
        }

        if self.permanent {
            terminal::print(
                MessageType::Warning,
                format!(
                    "Are you sure to remove {} permanently? (y/N): ",
                    self.name.display()
                ),
            )?;
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;

            return match buf.trim() {
                "y" | "Y" | "yes" | "Yes" | "YES" => match self.handle_type {
                    File => fs::remove_file(&self.name).map_err(|reason| {
                        (XiloError::RemoveFilePermanentlyFailed {
                            filename: self.name.clone(),
                            reason,
                        })
                        .into()
                    }),
                    Directory => fs::remove_dir_all(&self.name).map_err(|reason| {
                        (XiloError::RemoveDirPermanentlyFailed {
                            dirname: self.name.clone(),
                            reason,
                        })
                        .into()
                    }),
                },
                _ => Ok(()),
            };
        }

        let new_name = {
            let mut hasher = Sha256::new();
            hasher.update(self.name.to_string_lossy().as_bytes());
            hasher.update(format!("{:?}", SystemTime::now()).as_bytes());
            let hash = hasher.finalize();
            let base64_hash = Base64Url::encode_string(&hash);

            let mut tmp = self.trashbin_path.to_path_buf();
            tmp.push(format!(
                "{}!{}",
                base64_hash,
                self.name.file_name().unwrap().to_string_lossy()
            ));
            tmp
        };

        if !self.force {
            terminal::print(
                MessageType::Warning,
                format!("Are you sure to remove {}? (y/N): ", self.name.display()),
            )?;
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;

            match buf.trim() {
                "y" | "Y" | "yes" | "Yes" | "YES" => {}
                _ => return Ok(()),
            }
        }

        fs::rename(&self.name, new_name).map_err(|reason| {
            match self.handle_type {
                File => XiloError::RemoveFileFailed {
                    filename: self.name.clone(),
                    reason,
                },
                Directory => XiloError::RemoveDirFailed {
                    dirname: self.name.clone(),
                    reason,
                },
            }
            .into()
        })
    }
}
